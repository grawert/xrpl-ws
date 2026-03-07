use crate::subscriptions::handle::SubscriptionHandle;
pub mod config;
pub mod connection;
pub mod error;
pub mod request;
pub mod socket;
pub mod subscriptions;
pub mod types;

pub use config::ClientConfig;
pub use connection::{ConnectionPools, LoadBalancing, SubscriptionClass};
pub use error::XrplError;

use std::sync::Arc;

use dashmap::DashMap;
use serde_json::Value;
use tokio::sync::{broadcast, oneshot, Mutex};
use tokio::time::timeout;

use connection::ConnectionManager;
use request::{XrplRequest, XrplSubscription};
use socket::SocketCommand;

/// Main client for interacting with the XRP Ledger via WebSocket.
/// Handles connection management, requests, and subscriptions.
#[derive(Clone)]
pub struct XrplClient {
    pub url: String,
    config: ClientConfig,
    connection_manager: Arc<Mutex<ConnectionManager>>,
    events_tx: broadcast::Sender<Value>,
    subscriptions: Arc<DashMap<u64, Value>>,
}

impl XrplClient {
    /// Create a new client with the default configuration.
    pub async fn new(url: impl Into<String>) -> Result<Self, XrplError> {
        Self::with_config(url, ClientConfig::default()).await
    }

    /// Create a new client with a custom configuration.
    pub async fn with_config(
        url: impl Into<String>,
        config: ClientConfig,
    ) -> Result<Self, XrplError> {
        let url = url.into();
        let subscriptions = Arc::new(DashMap::new());

        let (connection_manager, events_tx) = ConnectionManager::new(
            url.clone(),
            config.clone(),
            subscriptions.clone(),
        )
        .await?;

        Ok(Self {
            url,
            config: config.clone(),
            connection_manager: Arc::new(Mutex::new(connection_manager)),
            events_tx,
            subscriptions,
        })
    }

    /// Send a raw JSON request to the XRP Ledger and return the response.
    async fn send_raw(&self, payload: Value) -> Result<Value, XrplError> {
        #[cfg(feature = "jsondump")]
        eprintln!("-- JSONDUMP REQUEST: {}", &payload);

        let (responder, rx) = oneshot::channel();
        let command = SocketCommand::Request { payload, responder };

        // Use Trading class for general requests as default
        let mut manager = self.connection_manager.lock().await;
        manager.send_command(command, SubscriptionClass::Trading).await?;
        drop(manager); // Release the lock early

        let response = timeout(self.config.request_timeout, rx)
            .await
            .map_err(|_| {
                XrplError::Timeout(
                    self.config.request_timeout.as_millis() as u64
                )
            })?
            .map_err(|_| XrplError::Disconnected)??;

        #[cfg(feature = "jsondump")]
        eprintln!("-- JSONDUMP RESPONSE : {}", &response);

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

    /// Send a typed XRPL request and deserialize the response.
    pub async fn request<T: XrplRequest>(
        &self,
        req: T,
    ) -> Result<T::Response, XrplError> {
        let value = self.send_raw(req.to_value()).await?;
        serde_json::from_value(value)
            .map_err(|e| XrplError::ParseError(e.to_string()))
    }

    /// Subscribe to a stream of XRPL events/messages for a given subscription type.
    pub async fn subscribe<
        T: XrplSubscription + Clone + Send + Sync + 'static,
    >(
        &self,
        sub: T,
    ) -> Result<(T::Response, SubscriptionHandle<T, T::Message>), XrplError>
    {
        let payload = sub.to_value();
        let subscription_class = sub.subscription_class();

        let key = sub.key();
        self.subscriptions.insert(key, payload.clone());

        // Send subscription request using the appropriate connection pool
        let (responder, rx) = oneshot::channel();
        let command = SocketCommand::Request { payload, responder };

        let mut manager = self.connection_manager.lock().await;
        manager.send_command(command, subscription_class).await?;
        drop(manager); // Release the lock early

        let response = timeout(self.config.request_timeout, rx)
            .await
            .map_err(|_| {
                XrplError::Timeout(
                    self.config.request_timeout.as_millis() as u64
                )
            })?
            .map_err(|_| XrplError::Disconnected)??;

        let response: T::Response = serde_json::from_value(response)
            .map_err(|e| XrplError::ParseError(e.to_string()))?;

        let raw_receiver = self.events_tx.subscribe();
        let typed_receiver = typed_receiver::<T>(
            raw_receiver,
            self.config.typed_receiver_channel_size,
        );

        let handle = SubscriptionHandle::new(
            Arc::new(self.clone()),
            sub,
            typed_receiver,
        );
        Ok((response, handle))
    }

    /// Unsubscribe from a subscription, given its parameters.
    pub async fn unsubscribe<T: XrplSubscription + Clone + Send + 'static>(
        &self,
        _params: &T,
    ) -> Result<(), XrplError> {
        Err(XrplError::ApiError { error: "Unsubscription objects have been removed. Unsubscribe is now managed by SubscriptionHandle.".to_string(), error_code: None, error_message: None })
    }
}

fn typed_receiver<T: XrplSubscription>(
    mut raw: broadcast::Receiver<Value>,
    channel_size: usize,
) -> broadcast::Receiver<T::Message> {
    let (tx, rx) = broadcast::channel(channel_size);
    tokio::spawn(async move {
        loop {
            match raw.recv().await {
                Ok(value) => {
                    if T::matches(&value) {
                        if let Ok(msg) =
                            serde_json::from_value::<T::Message>(value.clone())
                        {
                            let _ = tx.send(msg);
                        } else {
                            eprintln!("Xrpl Parse Error: {}", value);
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("Receiver lagged: skipped {} messages", n);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    });
    rx
}
