use serde::{Deserialize, Serialize};

use crate::request::{XrplRequest, XrplResponse, XrplSubscription};
use super::AccountTransactionMessage;

/// Selects which transaction stream to subscribe to.
#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
enum TransactionStream {
    /// Validated transactions only.
    #[default]
    Transactions,
    /// Validated transactions plus in-flight (not-yet-validated) transactions.
    TransactionsProposed,
}

/// Subscription request for all transaction stream events.
///
/// Use [`validated`](Self::validated) (default) for confirmed transactions or
/// [`proposed`](Self::proposed) to also receive in-flight transactions.
///
/// # Examples
///
/// ```no_run
/// use xrpl::Client;
/// use xrpl::subscriptions::TransactionsSubscription;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let client = Client::new("wss://xrplcluster.com");
///     let mut handle = client.subscription().await?;
///     let (_resp, mut stream) = handle.subscribe(&TransactionsSubscription::validated()).await?;
///     while let Ok(msg) = stream.recv().await {
///         println!("{}: {}", msg.hash, msg.engine_result);
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug, Serialize, Clone)]
pub struct TransactionsSubscription {
    /// The transaction stream to subscribe to.
    streams: [TransactionStream; 1],
}

impl TransactionsSubscription {
    /// Subscribe to the `transactions_proposed` stream: all validated transactions
    /// plus in-flight transactions that have not yet been included in a validated ledger.
    pub fn proposed() -> Self {
        Self { streams: [TransactionStream::TransactionsProposed] }
    }

    /// Subscribe to the `transactions` stream: validated transactions only.
    pub fn validated() -> Self {
        Self { streams: [TransactionStream::Transactions] }
    }
}

impl Default for TransactionsSubscription {
    /// Defaults to the [`validated`](Self::validated) stream (confirmed transactions only).
    fn default() -> Self {
        Self::validated()
    }
}

impl XrplRequest for TransactionsSubscription {
    type Response = XrplResponse<TransactionsSubscriptionResponse>;
    const COMMAND: &str = "subscribe";
}

impl XrplSubscription for TransactionsSubscription {
    type Message = AccountTransactionMessage;
    const MESSAGE_TYPE: &'static str = "transaction";
}

/// Initial response returned when subscribing to the transactions stream.
#[derive(Debug, Deserialize)]
pub struct TransactionsSubscriptionResponse {}
