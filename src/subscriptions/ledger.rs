use serde::{Deserialize, Serialize};

use crate::request::{XrplRequest, XrplResponse, XrplSubscription};

/// Subscription request for the `ledger` stream.
///
/// Sends a `ledgerClosed` message whenever the consensus process declares
/// a new validated ledger.
///
/// # Examples
///
/// ```no_run
/// use xrpl::Client;
/// use xrpl::subscriptions::LedgerSubscription;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let client = Client::new("wss://xrplcluster.com");
///     let mut session = client.subscription().await?;
///     let (_resp, mut stream) = session.subscribe(&LedgerSubscription::new()).await?;
///     while let Ok(msg) = stream.recv().await {
///         println!("ledger {} closed ({} txns)", msg.ledger_index, msg.txn_count);
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct LedgerSubscription {
    streams: [&'static str; 1],
}

impl LedgerSubscription {
    /// Creates a new subscription to the `ledger` stream.
    pub fn new() -> Self {
        Self { streams: [<Self as XrplSubscription>::STREAM] }
    }
}

impl Default for LedgerSubscription {
    fn default() -> Self {
        Self::new()
    }
}

impl XrplRequest for LedgerSubscription {
    type Response = XrplResponse<LedgerSubscriptionResponse>;
    const COMMAND: &str = "subscribe";
}

impl XrplSubscription for LedgerSubscription {
    type Message = LedgerMessage;
    const STREAM: &'static str = "ledger";
    const MESSAGE_TYPE: &'static str = "ledgerClosed";
}

/// Initial response when subscribing to the `ledger` stream.
///
/// Contains the same fields as [`LedgerMessage`], except `type` and `txn_count`.
#[derive(Debug, Deserialize)]
pub struct LedgerSubscriptionResponse {
    /// Base transaction fee in fee units.
    pub fee_base: i64,
    /// Fee units per transaction cost unit; omitted when XRPFees amendment is active.
    pub fee_ref: Option<i64>,
    /// Hash of the most recently validated ledger.
    pub ledger_hash: String,
    /// Sequence number of the most recently validated ledger.
    pub ledger_index: i64,
    /// Close time of the most recently validated ledger (seconds since Ripple epoch).
    pub ledger_time: i64,
    /// Network ID that identifies the XRPL network, when present.
    pub network_id: Option<u32>,
    /// Minimum XRP reserve for an account, in drops.
    pub reserve_base: i64,
    /// Additional XRP reserve per owned ledger object, in drops.
    pub reserve_inc: i64,
    /// Comma-separated ranges of ledger sequence numbers available on this server.
    pub validated_ledgers: Option<String>,
}

/// A `ledgerClosed` stream message, emitted on every validated ledger close.
///
/// # Examples
///
/// ```no_run
/// use xrpl::Client;
/// use xrpl::subscriptions::LedgerSubscription;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let client = Client::new("wss://xrplcluster.com");
///     let mut session = client.subscription().await?;
///     let (_resp, mut stream) = session.subscribe(&LedgerSubscription::new()).await?;
///     while let Ok(msg) = stream.recv().await {
///         println!("ledger {} closed ({} txns)", msg.ledger_index, msg.txn_count);
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct LedgerMessage {
    /// Base transaction fee in fee units.
    pub fee_base: i64,
    /// Omitted when the XRPFees amendment is enabled.
    pub fee_ref: Option<i64>,
    /// Hash of the closed ledger.
    pub ledger_hash: String,
    /// Sequence number of the closed ledger.
    pub ledger_index: i64,
    /// Close time of the ledger in seconds since the Ripple epoch.
    pub ledger_time: i64,
    /// Network ID that identifies the XRPL network, when present.
    pub network_id: Option<u32>,
    /// Minimum XRP reserve for an account, in drops.
    pub reserve_base: i64,
    /// Additional XRP reserve per owned ledger object, in drops.
    pub reserve_inc: i64,
    /// Number of transactions included in the closed ledger.
    pub txn_count: i64,
    /// Comma-separated ranges of ledger sequence numbers available on this server.
    pub validated_ledgers: Option<String>,
}
