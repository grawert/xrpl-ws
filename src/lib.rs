pub mod error;
pub mod request;
pub mod socket;
pub mod subscriptions;
pub mod types;

use std::sync::Arc;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::sync::broadcast;

use error::XrplError;
use socket::XrplSocket;
use request::{XrplRequest, XrplSubscription};

#[derive(Clone)]
pub struct XrplClient {
    pub url: String,
    socket: Arc<XrplSocket>,
}

impl XrplClient {
    pub async fn new(url: &str) -> Result<Self, XrplError> {
        let socket = XrplSocket::connect(url).await?;
        Ok(Self { url: url.into(), socket: Arc::new(socket) })
    }

    async fn send(&self, payload: Value) -> Result<Value, XrplError> {
        let response = self.socket.request(payload).await?;

        if let Some(error) = response.get("error") {
            return Err(XrplError::ApiError {
                error: error.as_str().unwrap_or("unknown").to_string(),
                error_code: response
                    .get("error_code")
                    .and_then(|c| c.as_i64())
                    .map(|c| c as i32),
                error_message: response
                    .get("error_message")
                    .and_then(|m| m.as_str())
                    .map(str::to_string),
            });
        }

        Ok(response)
    }

    pub async fn request<T: XrplRequest>(
        &self,
        req: T,
    ) -> Result<T::Response, XrplError> {
        let payload = req.to_value();
        let value = self.send(payload).await?;
        parse(value)
    }

    pub async fn subscribe<T: XrplSubscription>(
        &self,
        sub: T,
    ) -> Result<(T::Response, broadcast::Receiver<T::Message>), XrplError> {
        let raw_receiver = self.socket.subscribe();
        let payload = sub.to_value();

        // Track for automatic re-subscription after reconnect
        self.socket.track_subscription(payload.clone()).await;

        let value = self.send(payload).await?;
        let response = parse(value)?;
        let receiver = typed_receiver::<T>(raw_receiver);
        Ok((response, receiver))
    }

    pub fn is_connected(&self) -> bool {
        self.socket.is_connected()
    }
}

fn parse<T: DeserializeOwned>(value: Value) -> Result<T, XrplError> {
    serde_json::from_value(value)
        .map_err(|e| XrplError::ParseError(e.to_string()))
}

fn typed_receiver<T: XrplSubscription>(
    mut raw: broadcast::Receiver<Value>,
) -> broadcast::Receiver<T::Message> {
    let (tx, rx) = broadcast::channel(32);
    tokio::spawn(async move {
        while let Ok(value) = raw.recv().await {
            if value["type"] == T::message_type() {
                if let Ok(msg) = serde_json::from_value(value) {
                    let _ = tx.send(msg);
                }
            }
        }
    });
    rx
}
