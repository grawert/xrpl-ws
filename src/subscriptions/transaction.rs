use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::connection::SubscriptionClass;
use crate::request::{XrplRequest, XrplResponse, XrplSubscription};
use super::AccountTransactionMessage;

/// Subscription request for all transaction stream events.
#[derive(Debug, Serialize, Clone)]
pub struct TransactionsSubscription {
    pub streams: Vec<String>,
}

impl TransactionsSubscription {
    /// Create a new TransactionsSubscription for all transactions.
    pub fn new() -> Self {
        Self { streams: vec!["transactions".to_string()] }
    }

    pub fn proposed() -> Self {
        Self { streams: vec!["transactions_proposed".to_string()] }
    }
}

impl Default for TransactionsSubscription {
    fn default() -> Self {
        Self::new()
    }
}

impl XrplRequest for TransactionsSubscription {
    type Response = XrplResponse<TransactionsSubscriptionResponse>;
    const COMMAND: &str = "subscribe";
}

impl Hash for TransactionsSubscription {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Sort streams for deterministic hashing
        let mut sorted_streams = self.streams.clone();
        sorted_streams.sort();
        for stream in sorted_streams {
            stream.hash(state);
        }
    }
}

impl XrplSubscription for TransactionsSubscription {
    type Message = AccountTransactionMessage;

    fn matches(value: &Value) -> bool {
        value.get("type").and_then(|t| t.as_str()) == Some("transaction")
    }

    fn subscription_class(&self) -> SubscriptionClass {
        SubscriptionClass::Bulk
    }
}

#[derive(Debug, Deserialize)]
pub struct TransactionsSubscriptionResponse {
    pub fee_base: Option<i64>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<i64>,
}

