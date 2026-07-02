//! # XRPL Client Library
//!
//! Lightweight async WebSocket client for the XRP Ledger. Supports requests,
//! subscriptions, and automatic reconnection. Transaction signing and
//! serialization is delegated to external libraries.
//!
//! ## Installation
//!
//! ```toml
//! [dependencies]
//! xrpl-ws = "0.1"
//! ```
//!
//! ## Requests
//!
//! ```no_run
//! use xrpl::{Client, request::account_info::AccountInfoRequest};
//! use anyhow::Result;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let client = Client::new("wss://xrplcluster.com");
//!
//!     let request = AccountInfoRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
//!
//!     let response = client.request(&request).await?;
//!     println!("Account balance: {}", response.result()?.account_data.balance);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Subscriptions
//!
//! Use [`Client::subscription`] to open a shared connection, then call
//! [`SubscriptionHandle::subscribe`] to receive a stream of validated
//! transactions for a specific account. After each transaction,
//! [`util::available_balance`] returns the spendable balance after reserves.
//!
//! ```no_run
//! use xrpl::{Client, subscriptions::AccountTransactionsSubscription, util::available_balance};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = Client::new("wss://xrplcluster.com");
//!     let account = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";
//!
//!     let sub = AccountTransactionsSubscription::validated([account])?;
//!     let mut handle = client.subscription().await?;
//!     let (_resp, mut stream) = handle.subscribe(&sub).await?;
//!
//!     while let Ok(tx) = stream.recv().await {
//!         let balance = available_balance(&client, account).await?;
//!         println!("{} — {} — spendable: {} drops", tx.hash, tx.engine_result, balance);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! When processing incoming payments, always use [`types::HasTransactionMeta::delivered_amount`]
//! instead of the transaction's `Amount` field to guard against partial-payment attacks:
//!
//! ```no_run
//! use xrpl::{Client, subscriptions::AccountTransactionsSubscription};
//! use xrpl::types::HasTransactionMeta;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = Client::new("wss://xrplcluster.com");
//!     let account = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";
//!
//!     let sub = AccountTransactionsSubscription::validated([account])?;
//!     let mut handle = client.subscription().await?;
//!     let (_resp, mut stream) = handle.subscribe(&sub).await?;
//!
//!     while let Ok(tx) = stream.recv().await {
//!         if !tx.validated { continue; }
//!         if let Some(amount) = tx.delivered_amount() {
//!             println!("Received: {amount}");
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### `handle.recv()` vs `stream.recv()`
//!
//! [`SubscriptionHandle::subscribe`] returns a [`SubscriptionStream`] scoped
//! to that one subscription's message type. Call [`SubscriptionStream::recv`]
//! on it to receive only that subscription's messages, already deserialized
//! into the concrete type (e.g. [`subscriptions::LedgerMessage`]).
//!
//! The handle itself also has [`SubscriptionHandle::recv`], which reads from
//! a single unified channel carrying every message pushed over the shared
//! connection — regardless of which or how many subscriptions are open on
//! it — typed as the [`SubscriptionEvent`] enum. Reach for it when one loop
//! needs to react to several subscription types together; reach for the
//! typed stream when a task only cares about one.
//!
//! ```no_run
//! use xrpl::{Client, SubscriptionEvent};
//! use xrpl::subscriptions::{LedgerSubscription, TransactionsSubscription};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = Client::new("wss://xrplcluster.com");
//!     let mut handle = client.subscription().await?;
//!
//!     // Each subscribe() call opens its own typed stream over the same
//!     // shared connection; both streams are kept alive here so their
//!     // subscriptions survive a reconnect.
//!     let (_, mut ledgers) = handle.subscribe(&LedgerSubscription::new()).await?;
//!     let (_, _txs) = handle.subscribe(&TransactionsSubscription::validated()).await?;
//!
//!     // Typed stream: only ledger messages, already deserialized.
//!     tokio::spawn(async move {
//!         while let Ok(msg) = ledgers.recv().await {
//!             println!("[ledgers] {} closed", msg.ledger_index);
//!         }
//!     });
//!
//!     // Unified handle: every message pushed on the connection, tagged by type.
//!     while let Ok(event) = handle.recv().await {
//!         match event {
//!             SubscriptionEvent::Ledger(msg) => {
//!                 println!("[handle] ledger {} closed", msg.ledger_index);
//!             }
//!             SubscriptionEvent::Transaction(tx) => {
//!                 println!("[handle] tx {} ({})", tx.hash, tx.engine_result);
//!             }
//!             SubscriptionEvent::BookChanges(_) => {}
//!             _ => {}
//!         }
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Builders
//!
//! Use the builders in [`types::builders`] to construct transaction payloads.
//! Call `.fill(&client)` before `.build()` to auto-populate `Sequence`, `Fee`,
//! and `LastLedgerSequence` from the network.
//!
//! ```no_run
//! use xrpl::{Client, xrp, types::{PaymentFlag, builders::PaymentBuilder}};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = Client::new("wss://xrplcluster.com");
//!
//!     let payment = PaymentBuilder::new(
//!         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
//!         "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
//!         xrp!(1.99),
//!     )
//!     .with_flags(PaymentFlag::PartialPayment)
//!     .fill(&client)
//!     .await?
//!     .build()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! Transactions with time-based fields use the Ripple epoch (seconds since
//! 2000-01-01 UTC). Use [`time::ripple_now`] to avoid off-by-30-years errors:
//!
//! ```no_run
//! use xrpl::{Client, xrp, time::ripple_now, types::builders::CheckCreateBuilder};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = Client::new("wss://xrplcluster.com");
//!
//!     let check = CheckCreateBuilder::new(
//!         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
//!         "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
//!         xrp!(1.99),
//!     )
//!     .with_expiration(ripple_now() + 86_400) // expires in 24 hours
//!     .fill(&client)
//!     .await?
//!     .build()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! See [`types::builders`] for the full list of available transaction builders.
//!
//! ## Signing
//!
//! Signing and binary serialization are outside the scope of this library and
//! are intentionally delegated to purpose-built crates (e.g. `ripple-keypairs`,
//! `xrpl-mithril`). Implement the [`types::SigningContext`] trait on your wallet
//! type to bridge the two.
//!
//! The process follows the XRPL signing protocol: serialize the transaction to
//! binary (excluding the signature fields), prepend `HASH_PREFIX_TRANSACTION_SIGN`
//! (the "STX" prefix), sign the bytes, attach the signature, then serialize the
//! final blob for submission.
//!
//! ```ignore
//! use anyhow::Context;
//! use ripple_keypairs::{PrivateKey, PublicKey};
//! use xrpl_mithril::codec::serializer::serialize_json_object;
//! use xrpl_mithril::codec::signing::HASH_PREFIX_TRANSACTION_SIGN;
//! use xrpl::types::{Transaction, SigningContext};
//!
//! struct Wallet {
//!     public_key: PublicKey,
//!     private_key: PrivateKey,
//! }
//!
//! impl SigningContext for Wallet {
//!     type Error = anyhow::Error;
//!
//!     fn sign_transaction(&self, tx: &Transaction) -> Result<String, Self::Error> {
//!         let mut tx_json = serde_json::to_value(tx)?;
//!         tx_json["SigningPubKey"] = self.public_key.to_string().into();
//!
//!         let buf = {
//!             let map = tx_json.as_object().context("Transaction should be JSON object")?;
//!             let mut buf = Vec::new();
//!             serialize_json_object(map, &mut buf, true)?;
//!             buf
//!         };
//!
//!         let mut signing_bytes = Vec::with_capacity(4 + buf.len());
//!         signing_bytes.extend_from_slice(&HASH_PREFIX_TRANSACTION_SIGN);
//!         signing_bytes.extend_from_slice(&buf);
//!         let signature = self.private_key.sign(&signing_bytes);
//!         tx_json["TxnSignature"] = signature.to_string().into();
//!
//!         let map = tx_json.as_object().context("Transaction should be JSON object")?;
//!         let mut buf = Vec::new();
//!         serialize_json_object(map, &mut buf, false)?;
//!
//!         Ok(hex::encode(buf).to_uppercase())
//!     }
//! }
//! ```
//!
//! Once the wallet is wired up, pass it to [`types::builders::SubmitRequestBuilder`]
//! together with the built transaction. Signing happens inside `build()` and the
//! result goes straight to [`Client::request`]:
//!
//! ```ignore
//! use xrpl::{Client, xrp, types::builders::{PaymentBuilder, SubmitRequestBuilder}};
//!
//! let client = Client::new("wss://xrplcluster.com");
//! let wallet = Wallet { /* ... */ };
//!
//! let tx = PaymentBuilder::new(
//!     wallet.public_key.derive_address(),
//!     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
//!     xrp!(10),
//! )
//! .fill(&client)
//! .await?
//! .build()?;
//!
//! let req = SubmitRequestBuilder::new(&tx, &wallet).build()?;
//! let result = client.request(&req).await?;
//!
//! assert_eq!(result.result()?.engine_result, "tesSUCCESS");
//! ```

/// Client configuration (timeouts, channel sizes, reconnect backoff).
pub mod config;
/// Error types returned by the client.
pub mod error;
/// Subscription handle for receiving streamed messages.
pub mod handle;
/// Request types and response envelopes for all XRPL JSON-RPC commands.
pub mod request;
pub(crate) mod socket;
/// Subscription request types and streamed message types.
pub mod subscriptions;
/// Ripple-epoch time conversion utilities.
pub mod time;
/// Transaction, account-object, amount, and builder types.
pub mod types;
/// Account utility helpers (balance, sequence, existence, flags).
pub mod util;

// Public re-exports
pub use error::XrplError;
pub use config::ClientConfig;
pub use handle::{SubscriptionEvent, SubscriptionHandle, SubscriptionStream};

use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;

use serde_json::Value;
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;

use handle::ClosedFlag;
use socket::{request, subscribe, SocketRequest};
use request::XrplRequest;

/// Extracts an application-level error from a raw rippled response, if present.
fn api_error(response: &Value) -> Option<XrplError> {
    response.get("error").map(|error| XrplError::ApiError {
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
                response
                    .get("error_exception")
                    .and_then(|m| m.as_str())
                    .map(str::to_string)
            }),
    })
}

/// Main client for interacting with the XRP Ledger via WebSocket.
/// Handles connection management, requests, and subscriptions.
///
/// # Examples
///
/// ## Creating a new client
/// ```no_run
/// use xrpl::Client;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let client = Client::new("wss://xrplcluster.com");
///     Ok(())
/// }
/// ```
///
/// ## Sending a request
/// ```no_run
/// use xrpl::{Client, request::account_info::AccountInfoRequest};
/// use anyhow::Result;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = Client::new("wss://xrplcluster.com");
///     let req = AccountInfoRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
///     let response = client.request(&req).await?;
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
///     let sub = LedgerSubscription::new();
///     let mut handle = client.subscription().await?;
///     let (_resp, mut stream) = handle.subscribe(&sub).await?;
///
///     while let Ok(msg) = stream.recv().await {
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
///     let mut handle = client.subscription().await?;
///     let (_resp, mut stream) = handle.subscribe(&sub).await?;
///
///     while let Ok(tx) = stream.recv().await {
///         println!("Transaction {}: {} ({})",
///             tx.hash,
///             tx.tx_json.account,
///             tx.engine_result
///         );
///     }
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct Client {
    pub url: String,
    config: ClientConfig,
    connection: mpsc::Sender<SocketRequest>,
}

impl Client {
    /// Create a new client with the default configuration.
    ///
    /// Spawns a shared background task that maintains a persistent WebSocket
    /// connection for all one-shot requests. Requires an active Tokio runtime.
    pub fn new(url: impl AsRef<str>) -> Self {
        Self::with_config(url, ClientConfig::default())
    }

    /// Create a new client with a custom configuration.
    ///
    /// Spawns a shared background task that maintains a persistent WebSocket
    /// connection for all one-shot requests. Requires an active Tokio runtime.
    pub fn with_config(url: impl AsRef<str>, config: ClientConfig) -> Self {
        let url = url.as_ref().to_string();
        let connection = request(url.clone(), config.clone());
        Self { url, config, connection }
    }

    /// Send a request to the XRP Ledger and return the response.
    ///
    /// All requests from this client share a single persistent WebSocket
    /// connection. Concurrent calls are multiplexed by request ID.
    pub async fn request<T: XrplRequest>(
        &self,
        req: &T,
    ) -> Result<T::Response, XrplError> {
        let request = req.to_value();
        let (responder, rx) = oneshot::channel();

        self.connection
            .send(SocketRequest { request, responder })
            .await
            .map_err(|_| XrplError::Disconnected)?;

        let response = timeout(self.config.request_timeout, rx)
            .await
            .map_err(|_| {
                XrplError::Timeout(
                    self.config.request_timeout.as_millis() as u64
                )
            })?
            .map_err(|_| XrplError::Disconnected)??;

        if let Some(err) = api_error(&response) {
            return Err(err);
        }

        let result: T::Response = serde_json::from_value(response)
            .map_err(|e| XrplError::ParseError(e.to_string()))?;

        Ok(result)
    }

    /// Opens the shared connection backing one or more subscription streams.
    ///
    /// This only opens the connection — it does not itself subscribe to
    /// anything. Call [`SubscriptionHandle::subscribe`] one or more times on
    /// the returned handle to open individual typed streams.
    pub async fn subscription(
        &self,
    ) -> Result<SubscriptionHandle<SubscriptionEvent>, XrplError> {
        let (connection, receiver) =
            subscribe(self.url.clone(), self.config.clone());

        let stream = SubscriptionStream {
            receiver,
            closed: ClosedFlag::new(AtomicBool::new(false)),
            _phantom: PhantomData,
        };

        Ok(SubscriptionHandle {
            _connection: connection,
            config: self.config.clone(),
            stream,
            closed_flags: Vec::new(),
        })
    }
}
