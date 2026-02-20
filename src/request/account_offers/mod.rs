use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};
use crate::types::Amount;

#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct AccountOffersRequest {
    pub account: String,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<Value>,
    pub limit: Option<u32>,
    pub marker: Option<Value>,
}

impl XrplRequest for AccountOffersRequest {
    type Response = XrplResponse<AccountOffersResponse>;
    const COMMAND: &'static str = "account_offers";
}

#[derive(Debug, Deserialize)]
pub struct AccountOffersResponse {
    pub account: String,
    pub offers: Vec<AccountOffer>,
    pub ledger_current_index: Option<u32>,
    pub ledger_index: Option<u32>,
    pub ledger_hash: Option<String>,
    pub marker: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct AccountOffer {
    pub flags: u32,
    pub seq: u32,
    pub taker_gets: Amount,
    pub taker_pays: Amount,
    pub quality: String,
    pub expiration: Option<u64>,
}
