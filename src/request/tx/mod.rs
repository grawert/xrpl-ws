use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};
use crate::types::Transaction;

#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
pub struct TxRequest {
    pub tx_hash: Option<String>,
    pub ctid: Option<String>,
    pub binary: Option<bool>,
    pub min_ledger: Option<u32>,
    pub max_ledger: Option<u32>,
}

impl XrplRequest for TxRequest {
    type Response = XrplResponse<TxResponse>;
    const COMMAND: &'static str = "tx";
}

#[derive(Debug, Clone, Deserialize)]
pub struct TxResponse {
    pub close_time_iso: Option<String>,
    pub ctid: Option<String>,
    pub hash: String,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<u32>,
    pub meta: Option<Value>,
    pub tx_json: Transaction,
    pub validated: bool,
}
