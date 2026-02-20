use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};

#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct SubmitRequest {
    pub tx_blob: String,
    pub fail_hard: Option<bool>,
}

impl XrplRequest for SubmitRequest {
    type Response = XrplResponse<SubmitResponse>;
    const COMMAND: &'static str = "submit";
}

#[derive(Debug, Deserialize)]
pub struct SubmitResponse {
    pub engine_result: String,
    pub engine_result_code: i64,
    pub engine_result_message: String,
    pub tx_blob: String,
    pub accepted: bool,
    pub account_sequence_available: u32,
    pub account_sequence_next: u32,
    pub applied: bool,
    pub broadcast: bool,
    pub kept: bool,
    pub queued: bool,
    pub open_ledger_cost: String,
    pub validated_ledger_index: u32,
}
