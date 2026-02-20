use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};
use crate::types::AccountObject;

#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct AccountObjectsRequest {
    pub account: String,
    pub deletion_blockers_only: Option<bool>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<Value>,
    pub limit: Option<u32>,
    pub marker: Option<Value>,
    #[serde(rename = "type")]
    pub kind: Option<AccountObjectType>,
}

impl XrplRequest for AccountObjectsRequest {
    type Response = XrplResponse<AccountObjectsResponse>;
    const COMMAND: &'static str = "account_objects";
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountObjectType {
    Bridge,
    Check,
    DepositPreauth,
    Escrow,
    #[serde(rename = "mptoken")]
    MPToken,
    #[serde(rename = "nft_offer")]
    NFTokenOffer,
    #[serde(rename = "nft_page")]
    NFTokenPage,
    Offer,
    #[serde(rename = "payment_channel")]
    PayChannel,
    #[serde(rename = "state")]
    RippleState,
    SignerList,
    Ticket,
}

#[derive(Debug, Deserialize)]
pub struct AccountObjectsResponse {
    pub account: String,
    pub account_objects: Vec<AccountObject>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<u32>,
    pub ledger_current_index: Option<u32>,
    pub limit: Option<u32>,
    pub marker: Option<Value>,
    pub validated: Option<bool>,
}
