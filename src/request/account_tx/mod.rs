use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};

#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
pub struct AccountTxRequest {
    pub account: String,
    pub ledger_index_min: Option<i64>,
    pub ledger_index_max: Option<i64>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<Value>,
    pub binary: Option<bool>,
    pub forward: Option<bool>,
    pub limit: Option<u32>,
    pub marker: Option<Value>,
}

impl XrplRequest for AccountTxRequest {
    type Response = XrplResponse<AccountTxResponse>;
    const COMMAND: &'static str = "account_tx";
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountTxResponse {
    pub account: String,
    pub ledger_index_min: Option<i64>,
    pub ledger_index_max: Option<i64>,
    pub marker: Option<Value>,
    pub transactions: Vec<AccountTransaction>,
    pub validated: Option<bool>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountTransaction {
    pub meta: Option<Value>,
    pub tx_json: Value,
    pub validated: bool,
}
