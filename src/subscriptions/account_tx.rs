use serde::{Deserialize, Serialize};

use crate::request::{XrplRequest, XrplResponse, XrplSubscription};
use crate::types::{
    HasTransactionMeta, Transaction, TransactionMeta,
    validation::validate_address, builders::BuildError,
};

/// Selects which account stream to subscribe to and carries the account list.
#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
enum AccountStream {
    Proposed { accounts_proposed: Vec<String> },
    Validated { accounts: Vec<String> },
}

/// Subscription request for account transaction events.
///
/// # Examples
///
/// ```no_run
/// use xrpl::Client;
/// use xrpl::subscriptions::AccountTransactionsSubscription;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let client = Client::new("wss://xrplcluster.com");
///     let sub = AccountTransactionsSubscription::validated(
///         ["rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"],
///     )?;
///     let mut handle = client.subscription().await?;
///     let (_resp, mut stream) = handle.subscribe(&sub).await?;
///     while let Ok(msg) = stream.recv().await {
///         println!("{}: {}", msg.hash, msg.engine_result);
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug, Serialize, Clone)]
pub struct AccountTransactionsSubscription {
    #[serde(flatten)]
    stream: AccountStream,
}

impl AccountTransactionsSubscription {
    /// Subscribe to `accounts_proposed`: validated transactions plus in-flight
    /// transactions not yet included in a validated ledger.
    pub fn proposed<I, S>(accounts: I) -> Result<Self, BuildError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let acc: Vec<String> =
            accounts.into_iter().map(|s| s.as_ref().to_string()).collect();
        for addr in &acc {
            validate_address(addr)?;
        }
        Ok(Self { stream: AccountStream::Proposed { accounts_proposed: acc } })
    }

    /// Subscribe to `accounts`: validated transactions only.
    pub fn validated<I, S>(accounts: I) -> Result<Self, BuildError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let acc: Vec<String> =
            accounts.into_iter().map(|s| s.as_ref().to_string()).collect();
        for addr in &acc {
            validate_address(addr)?;
        }
        Ok(Self { stream: AccountStream::Validated { accounts: acc } })
    }
}

impl XrplRequest for AccountTransactionsSubscription {
    type Response = XrplResponse<AccountSubscriptionResponse>;
    const COMMAND: &str = "subscribe";
}

impl XrplSubscription for AccountTransactionsSubscription {
    type Message = AccountTransactionMessage;
    const MESSAGE_TYPE: &'static str = "transaction";
}

/// Initial response returned when subscribing to account transaction events.
#[derive(Debug, Deserialize)]
pub struct AccountSubscriptionResponse {
    /// Accounts enrolled in the validated-transactions stream, when applicable.
    pub accounts: Option<Vec<String>>,
    /// Accounts enrolled in the proposed-transactions stream, when applicable.
    pub accounts_proposed: Option<Vec<String>>,
}

/// A server-pushed message for a transaction that affects a subscribed account.
///
/// Received on both the `accounts` and `accounts_proposed` streams. The
/// `validated` flag distinguishes whether the transaction is in a closed,
/// immutable ledger (`true`) or is still in-flight (`false`).
///
/// # Examples
///
/// ```no_run
/// use xrpl::Client;
/// use xrpl::subscriptions::AccountTransactionsSubscription;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let client = Client::new("wss://xrplcluster.com");
///     let sub = AccountTransactionsSubscription::validated(
///         ["rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"],
///     )?;
///     let mut handle = client.subscription().await?;
///     let (_resp, mut stream) = handle.subscribe(&sub).await?;
///     while let Ok(msg) = stream.recv().await {
///         if msg.validated {
///             println!("{}: {}", msg.hash, msg.engine_result);
///         }
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct AccountTransactionMessage {
    /// ISO 8601 close time of the ledger, when available.
    pub close_time_iso: Option<String>,
    /// Transaction result code (e.g. `"tesSUCCESS"`, `"tecNO_DST"`).
    pub engine_result: String,
    /// Numeric form of the engine result code.
    pub engine_result_code: i32,
    /// Human-readable description of the engine result.
    pub engine_result_message: String,
    /// SHA-512Half hash that uniquely identifies the transaction.
    pub hash: String,
    /// Hash of the validated ledger that contains this transaction, when validated.
    pub ledger_hash: Option<String>,
    /// Sequence number of the validated ledger that contains this transaction.
    pub ledger_index: Option<u32>,
    /// Sequence number of the current open ledger (present when not yet validated).
    pub ledger_current_index: Option<u32>,
    /// Transaction metadata with affected nodes and delivered amount, when validated.
    pub meta: Option<TransactionMeta>,
    /// The full transaction object.
    pub tx_json: Transaction,
    /// `true` if the transaction is in a closed, immutable ledger.
    pub validated: bool,
    /// Compact Transaction Identifier for cross-network lookup, when present.
    pub ctid: Option<String>,
    /// Internal submission status string, when present.
    pub status: Option<String>,
}

impl HasTransactionMeta for AccountTransactionMessage {
    fn transaction_meta(&self) -> Option<&TransactionMeta> {
        self.meta.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_subscription_proposed_serializes_correct_field() {
        let accounts = vec![
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        ];
        let sub = AccountTransactionsSubscription::proposed(accounts)
            .expect("Should accept valid addresses");
        let json: serde_json::Value = serde_json::to_value(&sub).unwrap();

        assert_eq!(
            json["accounts_proposed"][0],
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
        );
        assert_eq!(
            json["accounts_proposed"][1],
            "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"
        );
        assert!(json.get("accounts").is_none());
    }

    #[test]
    fn test_account_subscription_validated_serializes_correct_field() {
        let accounts = vec![
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        ];
        let sub = AccountTransactionsSubscription::validated(accounts)
            .expect("Should accept valid addresses");
        let json: serde_json::Value = serde_json::to_value(&sub).unwrap();

        assert_eq!(json["accounts"][0], "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
        assert_eq!(json["accounts"][1], "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
        assert!(json.get("accounts_proposed").is_none());
    }

    #[test]
    fn test_account_subscription_new_invalid() {
        let accounts = vec!["not_an_address"];
        let result = AccountTransactionsSubscription::proposed(accounts);

        assert!(matches!(result, Err(BuildError::Validation(_))));
    }
}
