use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};

#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct AccountChannelsRequest {
    pub account: String,
    pub destination_account: Option<String>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<Value>,
    pub limit: Option<u32>,
    pub marker: Option<Value>,
}

impl XrplRequest for AccountChannelsRequest {
    type Response = XrplResponse<AccountChannelsResponse>;
    const COMMAND: &'static str = "account_channels";
}

#[derive(Debug, Deserialize)]
pub struct AccountChannelsResponse {
    pub account: String,
    pub channels: Vec<AccountChannel>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<Value>,
    pub validated: Option<bool>,
    pub marker: Option<Value>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct AccountChannel {
    pub account: String,
    pub amount: String,
    pub balance: String,
    pub channel_id: String,
    pub destination_account: String,
    pub settle_delay: u64,
    pub public_key: Option<String>,
    pub public_key_hex: Option<String>,
    pub expiration: Option<u64>,
    pub cancel_after: Option<u64>,
    pub source_tag: Option<u32>,
    pub destination_tag: Option<u32>,
}
