//! # XRPL WebSocket Client
//!
//! A high-performance, async WebSocket client library for interacting with the XRP Ledger.
//! Provides comprehensive support for XRPL API methods, real-time subscriptions, and
//! transaction building with full Rust type safety.
//!
//! ## Installation
//!
//! ```toml
//! [dependencies]
//! xrpl-ws = "0.1"
//! ```
//!
//! ## Quick Start
//!
//! ```no_run
//! use xrpl::{Client, request::account_info::AccountInfoRequest};
//! use anyhow::Result;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let client = Client::new("wss://xrplcluster.com");
//!
//!     let request = AccountInfoRequest {
//!         account: "rAccount...".to_string(),
//!         ..Default::default()
//!     };
//!
//!     let response = client.request(request).await?;
//!     println!("Account balance: {}", response.result()?.account_data.balance);
//!
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod error;
pub mod handle;
pub mod request;
pub mod socket;
pub mod subscriptions;
pub mod types;

// Public re-exports
pub use error::XrplError;
pub use config::ClientConfig;
pub use handle::ConnectionHandle;

use tokio::sync::oneshot;
use tokio::time::timeout;

use socket::{RequestActor, SubscriptionActor, SocketCommand};
use request::{XrplRequest, XrplSubscription};

/// Main client for interacting with the XRP Ledger via WebSocket.
/// Handles connection management, requests, and subscriptions.
///
/// # Examples
///
/// ## Creating a new client
/// ```rust
/// use xrpl::Client;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = Client::new("wss://xrplcluster.com");
///     Ok(())
/// }
/// ```
///
/// ## Sending a request
/// ```rust
/// use xrpl::{Client, request::account_info::AccountInfoRequest};
/// use anyhow::Result;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = Client::new("wss://xrplcluster.com");
///     let req = AccountInfoRequest {
///         account: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
///         ..Default::default()
///     };
///     let response = client.request(req).await?;
///     println!("Account info: {:?}", response);
///     Ok(())
/// }
/// ```
///
/// ## Subscribing to a stream
/// ```no_run
/// use xrpl::Client;
/// use xrpl::subscriptions::LedgerSubscription;
/// use anyhow::Result;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = Client::new("wss://xrplcluster.com");
///     let sub = LedgerSubscription::default();
///     let (_resp, mut handle) = client.subscribe(sub).await?;
///
///     while let Ok(msg) = handle.recv().await {
///         println!("Received: {:?}", msg);
///     }
///     Ok(())
/// }
/// ```
///
/// ## Subscribing to transactions
/// ```no_run
/// use xrpl::Client;
/// use xrpl::subscriptions::TransactionsSubscription;
/// use anyhow::Result;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = Client::new("wss://xrplcluster.com");
///     let sub = TransactionsSubscription::validated();
///     let (_resp, mut handle) = client.subscribe(sub).await?;
///
///     while let Ok(tx_msg) = handle.recv().await {
///         if tx_msg.validated {
///             println!("Transaction {}: {} ({})",
///                 tx_msg.hash,
///                 tx_msg.tx_json.account,
///                 tx_msg.engine_result
///             );
///         }
///     }
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct Client {
    pub url: String,
    config: ClientConfig,
}

impl Client {
    /// Create a new client with the default configuration.
    pub fn new(url: impl Into<String>) -> Self {
        Self::with_config(url, ClientConfig::default())
    }

    /// Create a new client with a custom configuration.
    pub fn with_config(url: impl Into<String>, config: ClientConfig) -> Self {
        Self { url: url.into(), config }
    }

    /// Send a request to the XRP Ledger and return the response.
    pub async fn request<T: XrplRequest>(
        &self,
        req: T,
    ) -> Result<T::Response, XrplError> {
        let payload = req.to_value();

        #[cfg(feature = "jsondump")]
        eprintln!("-- JSONDUMP REQUEST: {}", &payload);

        // Use lightweight RequestActor for one-shot request
        let response = RequestActor::spawn_request(
            self.url.clone(),
            self.config.clone(),
            payload,
        )
        .await?;

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
                    .map(str::to_string)
                    .or_else(|| {
                        // XRPL also uses 'error_exception' for detailed error messages
                        response
                            .get("error_exception")
                            .and_then(|m| m.as_str())
                            .map(str::to_string)
                    }),
            });
        }

        let result: T::Response = serde_json::from_value(response)
            .map_err(|e| XrplError::ParseError(e.to_string()))?;

        Ok(result)
    }

    /// Returns the subscription response and a connection handle for receiving messages.
    pub async fn subscribe<T>(
        &self,
        sub: T,
    ) -> Result<(T::Response, ConnectionHandle<T::Message>), XrplError>
    where
        T: XrplSubscription + Clone + Send + Sync + 'static,
        T::Message: serde::de::DeserializeOwned
            + Clone
            + Send
            + std::fmt::Debug
            + 'static,
    {
        // Spawn dedicated connection for this subscription
        let (connection, typed_receiver) =
            SubscriptionActor::spawn_subscription(
                self.url.clone(),
                self.config.clone(),
            );

        // Send subscription request
        let payload = sub.to_value();
        let (responder, rx) = oneshot::channel();
        let command = SocketCommand::Request { payload, responder };

        connection.send(command).await.map_err(|_| XrplError::Disconnected)?;

        // Wait for subscription response
        let response = timeout(self.config.request_timeout, rx)
            .await
            .map_err(|_| {
                XrplError::Timeout(
                    self.config.request_timeout.as_millis() as u64
                )
            })?
            .map_err(|_| XrplError::Disconnected)??;

        // Parse response
        let response: T::Response = serde_json::from_value(response)
            .map_err(|e| XrplError::ParseError(e.to_string()))?;

        // Create handle with dedicated connection
        let handle = ConnectionHandle::new(connection, typed_receiver);

        Ok((response, handle))
    }
}
