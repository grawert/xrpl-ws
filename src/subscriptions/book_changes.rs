use serde::{Deserialize, Serialize};

use crate::request::{XrplRequest, XrplResponse, XrplSubscription};

/// Subscription request for the `book_changes` stream.
///
/// Sends a `bookChanges` message on every validated ledger close, containing
/// a summary of all order book changes that occurred in that ledger.
///
/// # Examples
///
/// ```no_run
/// use xrpl::Client;
/// use xrpl::subscriptions::BookChangesSubscription;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let client = Client::new("wss://xrplcluster.com");
///     let mut handle = client.subscription().await?;
///     let (_resp, mut stream) = handle.subscribe(&BookChangesSubscription::default()).await?;
///     while let Ok(msg) = stream.recv().await {
///         println!("ledger {} had {} book changes", msg.ledger_index, msg.changes.len());
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct BookChangesSubscription {
    streams: [&'static str; 1],
}

impl BookChangesSubscription {
    pub fn new() -> Self {
        Self { streams: [<Self as XrplSubscription>::STREAM] }
    }
}

impl Default for BookChangesSubscription {
    fn default() -> Self {
        Self::new()
    }
}

impl XrplRequest for BookChangesSubscription {
    type Response = XrplResponse<BookChangesSubscriptionResponse>;
    const COMMAND: &str = "subscribe";
}

impl XrplSubscription for BookChangesSubscription {
    type Message = BookChangesMessage;
    const STREAM: &'static str = "book_changes";
    const MESSAGE_TYPE: &'static str = "bookChanges";
}

/// Initial response returned when subscribing to the `book_changes` stream.
#[derive(Debug, Deserialize)]
pub struct BookChangesSubscriptionResponse {
    /// Base transaction fee in fee units at the time of subscription.
    pub fee_base: Option<i64>,
    /// Hash of the most recently validated ledger at the time of subscription.
    pub ledger_hash: Option<String>,
    /// Sequence number of the most recently validated ledger.
    pub ledger_index: Option<i64>,
}

/// A `bookChanges` stream message, emitted on every validated ledger close.
#[derive(Debug, Clone, Deserialize)]
pub struct BookChangesMessage {
    /// Sequence number of the closed ledger.
    pub ledger_index: u64,
    /// Hash of the closed ledger.
    pub ledger_hash: String,
    /// Close time of the ledger in seconds since the Ripple epoch.
    pub ledger_time: u64,
    /// One entry for each order book that had activity in this ledger.
    pub changes: Vec<BookUpdate>,
}

/// One entry per order book that changed in the ledger.
///
/// `currency_a` and `currency_b` identify the pair as `"XRP_drops"` for XRP
/// or `"issuer/currency"` for issued currencies. All numeric fields are
/// string-encoded to preserve precision.
#[derive(Debug, Clone, Deserialize)]
pub struct BookUpdate {
    /// First asset in the pair (`"XRP_drops"` for XRP, `"issuer/currency"` for tokens).
    pub currency_a: String,
    /// Second asset in the pair.
    pub currency_b: String,
    /// Total amount of `currency_a` traded.
    pub volume_a: String,
    /// Total amount of `currency_b` traded.
    pub volume_b: String,
    /// Highest exchange rate seen in this ledger (currency_a per currency_b).
    pub high: String,
    /// Lowest exchange rate seen in this ledger.
    pub low: String,
    /// Opening exchange rate (first trade in this ledger).
    pub open: String,
    /// Closing exchange rate (last trade in this ledger).
    pub close: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_serializes_correct_stream() {
        let sub = BookChangesSubscription::default();
        let json = serde_json::to_value(&sub).unwrap();
        assert_eq!(json["streams"][0], "book_changes");
    }
}
