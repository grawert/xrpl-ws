use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};

#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct AccountLinesRequest {
    pub account: String,
    pub ignore_default: Option<bool>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<Value>,
    pub limit: Option<u32>,
    pub marker: Option<Value>,
    pub peer: Option<String>,
}

impl XrplRequest for AccountLinesRequest {
    type Response = XrplResponse<AccountLinesResponse>;
    const COMMAND: &'static str = "account_lines";
}

#[derive(Debug, Deserialize)]
pub struct AccountLinesResponse {
    pub account: String,
    pub lines: Vec<Trustline>,
    pub ledger_current_index: Option<u32>,
    pub ledger_index: Option<u32>,
    pub ledger_hash: Option<String>,
    pub marker: Option<Value>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct Trustline {
    pub account: String,
    pub balance: String,
    pub currency: String,
    pub limit: String,
    pub limit_peer: String,
    pub quality_in: i64,
    pub quality_out: i64,
    pub no_ripple: Option<bool>,
    pub no_ripple_peer: Option<bool>,
    pub authorized: Option<bool>,
    pub peer_authorized: Option<bool>,
    pub freeze: Option<bool>,
    pub freeze_peer: Option<bool>,
}
