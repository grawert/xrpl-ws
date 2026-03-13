use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_tungstenite::{
    connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream,
};

use crate::error::XrplError;
use crate::config::ClientConfig;

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    REQUEST_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug)]
pub enum SocketCommand {
    Request {
        payload: Value,
        responder: oneshot::Sender<Result<Value, XrplError>>,
    },
}

/// Simple actor for one-shot requests
pub struct RequestActor;

impl RequestActor {
    /// Spawn a lightweight connection for one-shot requests.
    /// Connection closes after response is received.
    pub async fn spawn_request(
        url: String,
        config: ClientConfig,
        payload: Value,
    ) -> Result<Value, XrplError> {
        let mut payload = payload;
        let id = next_id();
        payload["id"] = id.into();

        // Connect directly
        let (ws_stream, _) =
            connect_async(&url).await.map_err(|_| XrplError::Disconnected)?;

        let (mut write, mut read) = ws_stream.split();

        // Send request
        write
            .send(Message::Text(payload.to_string().into()))
            .await
            .map_err(|_| XrplError::Disconnected)?;

        // Wait for response with timeout
        let response = tokio::time::timeout(config.request_timeout, async {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(value) = serde_json::from_str::<Value>(&text)
                        {
                            if let Some(response_id) =
                                value.get("id").and_then(|id| id.as_u64())
                            {
                                if response_id == id {
                                    return Ok(value);
                                }
                            }
                        }
                    }
                    Ok(Message::Ping(data)) => {
                        let _ = write.send(Message::Pong(data)).await;
                    }
                    _ => break,
                }
            }
            Err(XrplError::Disconnected)
        })
        .await
        .map_err(|_| {
            XrplError::Timeout(config.request_timeout.as_millis() as u64)
        })??;

        Ok(response)
    }
}

/// Connection actor for persistent subscriptions with reconnection and replay.
pub struct SubscriptionActor<T>
where
    T: serde::de::DeserializeOwned + Clone + Send + 'static,
{
    url: String,
    config: ClientConfig,
    cmd_rx: mpsc::Receiver<SocketCommand>,
    typed_tx: broadcast::Sender<T>,
    pending: HashMap<u64, oneshot::Sender<Result<Value, XrplError>>>,
    subscription_payload: Option<Value>,
}

impl<T> SubscriptionActor<T>
where
    T: serde::de::DeserializeOwned + Clone + Send + 'static,
{
    /// Spawn a persistent connection for typed subscription messages.
    /// Handles reconnection and subscription replay automatically.
    pub fn spawn_subscription(
        url: String,
        config: ClientConfig,
    ) -> (mpsc::Sender<SocketCommand>, broadcast::Receiver<T>) {
        let (cmd_tx, cmd_rx) = mpsc::channel(config.cmd_channel_size);
        let (typed_tx, typed_rx) =
            broadcast::channel(config.subscription_channel_size);

        let actor = Self {
            url,
            config,
            cmd_rx,
            typed_tx,
            pending: HashMap::new(),
            subscription_payload: None,
        };

        tokio::spawn(async move {
            actor.run().await;
        });

        (cmd_tx, typed_rx)
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

            // Connection dropped, fail all pending requests
            for (_, responder) in self.pending.drain() {
                let _ = responder.send(Err(XrplError::Disconnected));
            }

            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(self.config.max_backoff);
        }
    }

    async fn handle_connection(
        &mut self,
        ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    ) {
        let (mut write, mut read) = ws_stream.split();
        let mut ping_interval =
            tokio::time::interval(self.config.keepalive_interval);

        // Re-apply active subscription immediately upon connection
        if let Some(ref subscription) = self.subscription_payload {
            let _ = write
                .send(Message::Text(subscription.to_string().into()))
                .await;
        }

        loop {
            tokio::select! {
                _ = ping_interval.tick() => {
                    if write.send(Message::Ping(vec![].into())).await.is_err() {
                        break;
                    }
                }

                Some(cmd) = self.cmd_rx.recv() => {
                    match cmd {
                        SocketCommand::Request { mut payload, responder } => {
                            let id = next_id();
                            payload["id"] = id.into();

                            // If this is a subscription request, store it for replay on reconnect
                            if payload.get("command").and_then(|c| c.as_str()) == Some("subscribe") {
                                self.subscription_payload = Some(payload.clone());
                            }

                            if write.send(Message::Text(payload.to_string().into())).await.is_err() {
                                let _ = responder.send(Err(XrplError::Disconnected));
                                break;
                            }
                            self.pending.insert(id, responder);
                        }
                    }
                }

                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => self.handle_incoming_text(text.to_string()),
                        Some(Ok(Message::Ping(data))) => {
                            let _ = write.send(Message::Pong(data)).await;
                        }
                        Some(Err(_)) | None => break, // Disconnected
                        _ => {}
                    }
                }
            }
        }
    }

    fn handle_incoming_text(&mut self, text: String) {
        match serde_json::from_str::<Value>(&text) {
            Ok(value) => {
                if let Some(id) = value.get("id").and_then(|id| id.as_u64()) {
                    // Response message - route to pending request
                    if let Some(responder) = self.pending.remove(&id) {
                        let _ = responder.send(Ok(value));
                    }
                } else {
                    // Subscription event - deserialize directly to T and send
                    if let Ok(typed_msg) =
                        serde_json::from_value::<T>(value.clone())
                    {
                        let _ = self.typed_tx.send(typed_msg);
                    } else {
                        eprintln!(
                            "Failed to deserialize subscription message to expected type: {}",
                            value
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "Failed to parse incoming JSON: {} - Raw text: {}",
                    e, text
                );
            }
        }
    }
}
