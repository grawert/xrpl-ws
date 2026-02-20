use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::request::{XrplRequest, XrplResponse, XrplSubscription};

#[derive(Debug, Default, Serialize)]
pub struct LedgerSubscription;

impl XrplRequest for LedgerSubscription {
    type Response = XrplResponse<LedgerSubscriptionResponse>;
    const COMMAND: &'static str = "subscribe";

    fn to_value(&self) -> Value {
        json!({
            "id": Uuid::new_v4().to_string(),
            "command": "subscribe",
            "streams": ["ledger"],
            "api_version": Self::API_VERSION,
        })
    }
}

impl XrplSubscription for LedgerSubscription {
    type Message = LedgerMessage;
    fn message_type() -> &'static str {
        "ledgerClosed"
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

#[derive(Debug, Default, Serialize)]
pub struct LedgerUnsubscription;

impl XrplRequest for LedgerUnsubscription {
    type Response = XrplResponse<UnsubscribeResponse>;
    const COMMAND: &'static str = "unsubscribe";

    fn to_value(&self) -> Value {
        json!({
            "id": Uuid::new_v4().to_string(),
            "command": "unsubscribe",
            "streams": ["ledger"],
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct UnsubscribeResponse {}
