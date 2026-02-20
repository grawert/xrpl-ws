use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};

#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct AccountCurrenciesRequest {
    pub account: String,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<Value>,
}

impl XrplRequest for AccountCurrenciesRequest {
    type Response = XrplResponse<AccountCurrenciesResponse>;
    const COMMAND: &'static str = "account_currencies";
}

#[derive(Debug, Deserialize)]
pub struct AccountCurrenciesResponse {
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<u32>,
    pub receive_currencies: Vec<String>,
    pub send_currencies: Vec<String>,
    pub validated: bool,
}
