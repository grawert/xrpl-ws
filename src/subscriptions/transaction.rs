use serde::{Deserialize, Serialize};

use crate::request::{XrplRequest, XrplResponse, XrplSubscription};
use super::AccountTransactionMessage;

/// Subscription request for all transaction stream events.
#[derive(Debug, Serialize, Clone)]
pub struct TransactionsSubscription {
    pub streams: Vec<String>,
}

impl TransactionsSubscription {
    /// Subscribe to validated and unvalidated transactions.
    pub fn proposed() -> Self {
        Self { streams: vec!["transactions_proposed".to_string()] }
    }

    /// Subscribe to validated only transactions.
    pub fn validated() -> Self {
        Self { streams: vec!["transactions".to_string()] }
    }
}

impl Default for TransactionsSubscription {
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
}

#[derive(Debug, Deserialize)]
pub struct TransactionsSubscriptionResponse {
    pub fee_base: Option<i64>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<i64>,
}
