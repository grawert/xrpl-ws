use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};
use crate::types::Amount;

/// Retrieves all sell offers for a specific NFToken.
#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct NftSellOffersRequest {
    /// 64-character hex NFToken ID to query sell offers for.
    pub nft_id: String,
    /// Ledger hash to target a specific ledger version.
    pub ledger_hash: Option<String>,
    /// Ledger index or shortcut ("validated", "closed", "current").
    pub ledger_index: Option<Value>,
    /// Maximum number of offers per page.
    pub limit: Option<u32>,
    /// Opaque pagination cursor from a previous response; omit for the first page.
    pub marker: Option<Value>,
}

impl XrplRequest for NftSellOffersRequest {
    type Response = XrplResponse<NftSellOffersResponse>;
    const COMMAND: &str = "nft_sell_offers";
}

/// Response to an `nft_sell_offers` request.
#[derive(Debug, Deserialize)]
pub struct NftSellOffersResponse {
    /// NFToken ID the offers are for.
    pub nft_id: String,
    /// Sell offers for the NFToken.
    pub offers: Vec<NftOffer>,
    /// Limit the number of NFT sell offers to retrieve.
    pub limit: Option<u32>,
    /// Sequence number of the current open ledger (unvalidated results).
    pub ledger_current_index: Option<u32>,
    /// Sequence number of the ledger version used.
    pub ledger_index: Option<u32>,
    /// Hash of the ledger version used.
    pub ledger_hash: Option<String>,
    /// Whether the data comes from a validated ledger.
    pub validated: Option<bool>,
    /// Opaque pagination cursor; present when more pages are available.
    pub marker: Option<Value>,
}

/// An NFToken offer returned by `nft_buy_offers` and `nft_sell_offers`.
#[derive(Debug, Deserialize)]
pub struct NftOffer {
    /// Offered or asking amount for the NFToken.
    pub amount: Amount,
    /// Offer flags bit field.
    pub flags: u32,
    /// Ledger index (ID) of the NFTokenOffer object.
    pub nft_offer_index: String,
    /// Account that placed the offer.
    pub owner: String,
}
