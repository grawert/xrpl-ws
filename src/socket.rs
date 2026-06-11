use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_tungstenite::{
    connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream,
};

use crate::config::ClientConfig;
use crate::error::XrplError;

#[cfg(feature = "jsondump")]
macro_rules! json_dump {
    ($label:expr, $value:expr) => {
        json_dump!($label, $value, None::<u64>)
    };
    ($label:expr, $value:expr, $id:expr) => {
        if let Ok(json) = serde_json::to_string_pretty($value) {
            use std::io::Write;
            let mut stderr = std::io::stderr().lock();
            let id_str = $id.map(|i| format!(" #{}", i)).unwrap_or_default();
            let _ =
                writeln!(stderr, "\n=== {}{}\n{}\n===\n", $label, id_str, json);
        }
    };
}

#[cfg(not(feature = "jsondump"))]
macro_rules! json_dump {
    ($label:expr, $value:expr) => {};
    ($label:expr, $value:expr, $id:expr) => {};
}

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    REQUEST_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug)]
pub(crate) struct SocketRequest {
    pub(crate) payload: Value,
    pub(crate) responder: oneshot::Sender<Result<Value, XrplError>>,
}

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

trait SessionHandler: Send + 'static {
    fn on_connect(&mut self) -> Vec<Value>;
    fn on_request(&mut self, req: SocketRequest) -> Option<Value>;
    fn on_message(&mut self, value: Value);
    fn on_disconnect(&mut self);
}

struct ReconnectLoop<H: SessionHandler> {
    url: String,
    config: ClientConfig,
    req_rx: mpsc::Receiver<SocketRequest>,
    handler: H,
}

impl<H: SessionHandler> ReconnectLoop<H> {
    fn new(
        url: String,
        config: ClientConfig,
        req_rx: mpsc::Receiver<SocketRequest>,
        handler: H,
    ) -> Self {
        Self { url, config, req_rx, handler }
    }

    async fn run(mut self) {
        let mut backoff = self.config.initial_backoff;

        loop {
            match connect_async(&self.url).await {
                Ok((ws_stream, _)) => {
                    backoff = self.config.initial_backoff;
                    self.handle_connection(ws_stream).await;
                }
                Err(e) => {
                    eprintln!(
                        "WS Connect Error: {e}. Retrying in {}s",
                        backoff.as_secs()
                    );
                }
            }

            self.handler.on_disconnect();
            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(self.config.max_backoff);
        }
    }

    async fn handle_connection(&mut self, ws_stream: WsStream) {
        let (mut write, mut read) = ws_stream.split();
        let mut ping_interval =
            tokio::time::interval(self.config.keepalive_interval);

        for payload in self.handler.on_connect() {
            let _ = write.send(Message::Text(payload.to_string().into())).await;
        }

        loop {
            tokio::select! {
                _ = ping_interval.tick() => {
                    if write.send(Message::Ping(vec![].into())).await.is_err() {
                        break;
                    }
                }

                req = self.req_rx.recv() => {
                    match req {
                        Some(req) => {
                            if let Some(payload) = self.handler.on_request(req)
                                && write.send(Message::Text(payload.to_string().into())).await.is_err()
                            {
                                break;
                            }
                        }
                        None => break,
                    }
                }

                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            match serde_json::from_str::<Value>(&text) {
                                Ok(value) => self.handler.on_message(value),
                                Err(e) => eprintln!("Failed to parse incoming JSON: {e} - Raw: {text}"),
                            }
                        }
                        Some(Ok(Message::Ping(data))) => {
                            let _ = write.send(Message::Pong(data)).await;
                        }
                        Some(Err(_)) | None => break,
                        _ => {}
                    }
                }
            }
        }
    }
}

struct SubscriptionSession<T> {
    payload: Option<Value>,
    pending: HashMap<u64, oneshot::Sender<Result<Value, XrplError>>>,
    stream_tx: broadcast::Sender<T>,
}

impl<T> SessionHandler for SubscriptionSession<T>
where
    T: serde::de::DeserializeOwned + Clone + Send + 'static,
{
    fn on_connect(&mut self) -> Vec<Value> {
        self.payload.iter().cloned().collect()
    }

    fn on_request(&mut self, req: SocketRequest) -> Option<Value> {
        let SocketRequest { mut payload, responder } = req;
        let id = next_id();
        payload["id"] = id.into();

        json_dump!("REQUEST", &payload, Some(id));

        if let Some("subscribe") =
            payload.get("command").and_then(|c| c.as_str())
        {
            self.payload = Some(payload.clone());
        }

        self.pending.insert(id, responder);
        Some(payload)
    }

    fn on_message(&mut self, value: Value) {
        match value.get("id").and_then(|id| id.as_u64()) {
            Some(id) => {
                json_dump!("RESPONSE", &value, Some(id));
                if let Some(responder) = self.pending.remove(&id) {
                    let _ = responder.send(Ok(value));
                }
            }
            None => {
                json_dump!("PUSH_MESSAGE", &value);
                match serde_json::from_value::<T>(value.clone()) {
                    Ok(typed_msg) => {
                        let _ = self.stream_tx.send(typed_msg);
                    }
                    Err(_) => eprintln!(
                        "Failed to deserialize subscription message: {value}"
                    ),
                }
            }
        }
    }

    fn on_disconnect(&mut self) {
        for (_, responder) in self.pending.drain() {
            let _ = responder.send(Err(XrplError::Disconnected));
        }
    }
}

struct RequestSession {
    pending: HashMap<u64, oneshot::Sender<Result<Value, XrplError>>>,
}

impl SessionHandler for RequestSession {
    fn on_connect(&mut self) -> Vec<Value> {
        vec![]
    }

    fn on_request(&mut self, req: SocketRequest) -> Option<Value> {
        let SocketRequest { mut payload, responder } = req;
        let id = next_id();
        payload["id"] = id.into();
        json_dump!("REQUEST", &payload, Some(id));
        self.pending.insert(id, responder);
        Some(payload)
    }

    fn on_message(&mut self, value: Value) {
        if let Some(id) = value.get("id").and_then(|id| id.as_u64()) {
            json_dump!("RESPONSE", &value, Some(id));
            if let Some(responder) = self.pending.remove(&id) {
                let _ = responder.send(Ok(value));
            }
        }
    }

    fn on_disconnect(&mut self) {
        for (_, responder) in self.pending.drain() {
            let _ = responder.send(Err(XrplError::Disconnected));
        }
    }
}

/// Spawns a persistent, multiplexed WebSocket connection for one-shot requests.
/// Returns a sender that routes each [`SocketRequest`] through the shared connection.
pub(crate) fn request_pool(
    url: String,
    config: ClientConfig,
) -> mpsc::Sender<SocketRequest> {
    let (req_tx, req_rx) = mpsc::channel(config.cmd_channel_size);
    let session = RequestSession { pending: HashMap::new() };
    tokio::spawn(ReconnectLoop::new(url, config, req_rx, session).run());
    req_tx
}

pub(crate) fn subscribe<T>(
    url: String,
    config: ClientConfig,
) -> (mpsc::Sender<SocketRequest>, broadcast::Receiver<T>)
where
    T: serde::de::DeserializeOwned + Clone + Send + 'static,
{
    let (req_tx, req_rx) = mpsc::channel(config.cmd_channel_size);
    let (stream_tx, stream_rx) =
        broadcast::channel(config.subscription_channel_size);

    let session = SubscriptionSession {
        stream_tx,
        pending: HashMap::new(),
        payload: None,
    };

    tokio::spawn(ReconnectLoop::new(url, config, req_rx, session).run());

    (req_tx, stream_rx)
}
