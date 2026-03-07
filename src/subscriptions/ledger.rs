use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::connection::SubscriptionClass;
use crate::request::{XrplRequest, XrplResponse, XrplSubscription};

/// Subscription handle for ledger close events on the XRPL.
#[derive(Debug, Serialize, Clone)]
pub struct LedgerSubscription {
    pub streams: Vec<String>,
}
impl LedgerSubscription {
    /// Create a new LedgerSubscription for ledger stream updates.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for LedgerSubscription {
    fn default() -> Self {
        Self { streams: vec!["ledger".to_string()] }
    }
}

impl XrplRequest for LedgerSubscription {
    type Response = XrplResponse<LedgerSubscriptionResponse>;
    const COMMAND: &str = "subscribe";
}

impl Hash for LedgerSubscription {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Ledger subscription has no variable data, just hash the stream name
        "ledger".hash(state);
    }
}

impl XrplSubscription for LedgerSubscription {
    type Message = LedgerMessage;

    fn matches(value: &Value) -> bool {
        value.get("type").and_then(|t| t.as_str()) == Some("ledgerClosed")
    }

    fn subscription_class(&self) -> SubscriptionClass {
        SubscriptionClass::Priority
    }
}

#[derive(Debug, Deserialize)]
pub struct LedgerSubscriptionResponse {
    pub fee_base: i64,
    pub ledger_hash: String,
    pub ledger_index: i64,
    pub ledger_time: i64,
    pub reserve_base: i64,
    pub reserve_inc: i64,
    pub validated_ledgers: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LedgerMessage {
    pub fee_base: i64,
    pub ledger_hash: String,
    pub ledger_index: i64,
    pub ledger_time: i64,
    pub reserve_base: i64,
    pub reserve_inc: i64,
    pub txn_count: i64,
    #[serde(rename = "type")]
    pub kind: String,
    pub validated_ledgers: Option<String>,
}

