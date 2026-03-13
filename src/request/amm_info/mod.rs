use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};
use crate::types::Amount;

#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
pub struct AmmInfoRequest {
    pub account: Option<String>,
    pub amm_account: Option<String>,
    pub asset: Option<Amount>,
    pub asset2: Option<Amount>,
    pub ledger_index: Option<Value>,
    pub ledger_hash: Option<String>,
}

impl XrplRequest for AmmInfoRequest {
    type Response = XrplResponse<AmmInfoResponse>;
    const COMMAND: &str = "amm_info";
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthAccount {
    pub account: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuctionSlot {
    pub account: String,
    pub auth_accounts: Option<Vec<AuthAccount>>,
    pub discounted_fee: u32,
    pub expiration: String,
    pub price: Amount,
    pub time_interval: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VoteSlot {
    pub account: String,
    pub trading_fee: u32,
    pub vote_weight: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AmmDescription {
    pub account: String,
    pub amount: Amount,
    pub amount2: Amount,
    #[serde(default)]
    pub asset_frozen: Option<bool>,
    #[serde(default)]
    pub asset2_frozen: Option<bool>,
    pub auction_slot: Option<AuctionSlot>,
    pub lp_token: Amount,
    pub trading_fee: u32,
    pub vote_slots: Option<Vec<VoteSlot>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AmmInfoResponse {
    pub amm: AmmDescription,
    pub ledger_current_index: Option<u32>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<u32>,
    pub validated: Option<bool>,
}
