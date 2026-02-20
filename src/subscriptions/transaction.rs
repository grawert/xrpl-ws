use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::request::{XrplRequest, XrplResponse, XrplSubscription};
use crate::types::Transaction;

use super::ledger::UnsubscribeResponse;

#[derive(Debug, Serialize)]
pub struct AccountTransactionsSubscription {
    pub accounts: Vec<String>,
}

impl AccountTransactionsSubscription {
    pub fn new(accounts: Vec<String>) -> Self {
        Self { accounts }
    }
}

impl XrplRequest for AccountTransactionsSubscription {
    type Response = XrplResponse<AccountSubscriptionResponse>;
    const COMMAND: &'static str = "subscribe";

    fn to_value(&self) -> Value {
        json!({
            "id": Uuid::new_v4().to_string(),
            "command": "subscribe",
            "accounts": self.accounts,
            "api_version": Self::API_VERSION,
        })
    }
}

impl XrplSubscription for AccountTransactionsSubscription {
    type Message = AccountTransactionMessage;
    fn message_type() -> &'static str {
        "transaction"
    }
}

#[derive(Debug, Deserialize)]
pub struct AccountSubscriptionResponse {}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountTransactionMessage {
    pub close_time_iso: Option<String>,
    #[serde(rename = "type")]
    pub kind: String,
    pub engine_result: String,
    pub engine_result_code: i32,
    pub engine_result_message: String,
    pub hash: String,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<u32>,
    pub meta: Option<TransactionMeta>,
    pub tx_json: Transaction,
    pub validated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TransactionMeta {
    pub affected_nodes: Vec<Value>,
    pub transaction_index: u32,
    pub transaction_result: String,
    #[serde(rename = "delivered_amount")]
    pub delivered_amount: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct AccountTransactionsUnsubscription {
    pub accounts: Vec<String>,
}

impl AccountTransactionsUnsubscription {
    pub fn new(accounts: Vec<String>) -> Self {
        Self { accounts }
    }
}

impl XrplRequest for AccountTransactionsUnsubscription {
    type Response = XrplResponse<UnsubscribeResponse>;
    const COMMAND: &'static str = "unsubscribe";

    fn to_value(&self) -> Value {
        json!({
            "id": Uuid::new_v4().to_string(),
            "command": "unsubscribe",
            "accounts": self.accounts,
        })
    }
}
