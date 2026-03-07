use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use dashmap::DashMap;
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

pub struct ConnectionActor {
    url: String,
    config: ClientConfig,
    cmd_rx: mpsc::Receiver<SocketCommand>,
    events_tx: broadcast::Sender<Value>,
    subscriptions: Arc<DashMap<u64, Value>>,
    pending: HashMap<u64, oneshot::Sender<Result<Value, XrplError>>>,
}

impl ConnectionActor {
    pub fn spawn(
        url: String,
        subscriptions: Arc<DashMap<u64, Value>>,
        config: ClientConfig,
    ) -> (mpsc::Sender<SocketCommand>, broadcast::Sender<Value>) {
        let (cmd_tx, cmd_rx) = mpsc::channel(config.cmd_channel_size);
        let (events_tx, _) = broadcast::channel(config.events_channel_size);

        let mut actor = Self {
            url,
            config,
            cmd_rx,
            events_tx: events_tx.clone(),
            subscriptions,
            pending: HashMap::new(),
        };

        tokio::spawn(async move {
            actor.run().await;
        });

        (cmd_tx, events_tx)
    }

    async fn run(&mut self) {
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

        // Re-apply active subscriptions immediately upon connection
        for entry in self.subscriptions.iter() {
            let _ = write
                .send(Message::Text(entry.value().to_string().into()))
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
                    // Subscription event - broadcast to all listeners
                    let _ = self.events_tx.send(value);
                }
            }
            Err(_) => {} // Ignore malformed JSON
        }
    }
}
