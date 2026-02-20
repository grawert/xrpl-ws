use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};

#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct AccountInfoRequest {
    pub account: String,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<Value>,
    pub queue: Option<bool>,
    pub signer_lists: Option<bool>,
    pub strict: Option<bool>,
}

impl XrplRequest for AccountInfoRequest {
    type Response = XrplResponse<AccountInfoResponse>;
    const COMMAND: &'static str = "account_info";
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountInfoResponse {
    pub account_data: AccountRoot,
    pub signer_lists: Option<Vec<String>>,
    pub ledger_current_index: Option<u32>,
    pub ledger_index: Option<u32>,
    pub queue_data: Option<QueueData>,
    pub validated: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccountRoot {
    pub account: String,
    pub balance: String,
    pub flags: u32,
    pub ledger_entry_type: String,
    pub owner_count: u32,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: String,
    pub previous_txn_lgr_seq: u32,
    pub sequence: u32,
    #[serde(rename = "index")]
    pub index: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueueData {
    pub txn_count: u32,
    pub auth_change_queued: Option<bool>,
    pub lowest_sequence: Option<u32>,
    pub highest_sequence: Option<u32>,
    pub max_spend_drops_total: Option<String>,
    pub transactions: Option<Vec<QueueTransaction>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueueTransaction {
    pub auth_change: bool,
    pub fee: String,
    pub fee_level: String,
    pub max_spend_drops: String,
    pub seq: u32,
}
