use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};
use crate::types::{Amount, Asset};

/// Retrieves the current state of an Automated Market Maker (AMM) pool.
///
/// Identify the pool either by its `amm_account` address or by the `asset`/`asset2` pair.
///
/// # Example
/// ```rust
/// use xrpl::request::amm_info::AmmInfoRequest;
/// use xrpl::types::Asset;
///
/// let request = AmmInfoRequest {
///     asset: Some(Asset::xrp()),
///     asset2: Some(Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap()),
///     ledger_index: Some("validated".into()),
///     ..Default::default()
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
pub struct AmmInfoRequest {
    /// LP account to filter vote slots and auction slot by.
    pub account: Option<String>,
    /// AMM pool account address.
    pub amm_account: Option<String>,
    /// First asset in the pool pair (currency identifier only, no value).
    pub asset: Option<Asset>,
    /// Second asset in the pool pair (currency identifier only, no value).
    pub asset2: Option<Asset>,
    /// Ledger index or shortcut ("validated", "closed", "current").
    pub ledger_index: Option<Value>,
    /// Ledger hash to target a specific ledger version.
    pub ledger_hash: Option<String>,
}

impl AmmInfoRequest {
    /// Creates a new request to fetch an AMM by its specific account address.
    ///
    /// # Example
    /// ```rust
    /// use xrpl::request::amm_info::AmmInfoRequest;
    /// let req = AmmInfoRequest::by_account("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
    /// ```
    pub fn by_account(amm_account: impl Into<String>) -> Self {
        Self { amm_account: Some(amm_account.into()), ..Default::default() }
    }

    /// Creates a new request to fetch an AMM by its asset pair.
    ///
    /// # Example
    /// ```rust
    /// use xrpl::request::amm_info::AmmInfoRequest;
    /// use xrpl::types::Asset;
    /// let req = AmmInfoRequest::by_assets(
    ///     Asset::xrp(),
    ///     Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap()
    /// );
    /// ```
    pub fn by_assets(
        asset: impl Into<Asset>,
        asset2: impl Into<Asset>,
    ) -> Self {
        Self {
            asset: Some(asset.into()),
            asset2: Some(asset2.into()),
            ..Default::default()
        }
    }
}

impl XrplRequest for AmmInfoRequest {
    type Response = XrplResponse<AmmInfoResponse>;
    const COMMAND: &str = "amm_info";
}

/// An account authorized to trade at the discounted fee during the active auction slot.
#[derive(Debug, Clone, Deserialize)]
pub struct AuthAccount {
    /// Authorized account address.
    pub account: String,
}

/// The active auction slot held by an LP, granting a discounted trading fee.
#[derive(Debug, Clone, Deserialize)]
pub struct AuctionSlot {
    /// Account holding the auction slot.
    pub account: String,
    /// Additional accounts authorized to trade at the discounted fee.
    pub auth_accounts: Option<Vec<AuthAccount>>,
    /// Trading fee the slot holder pays, in units of 1/100,000.
    pub discounted_fee: u32,
    /// ISO 8601 expiration time of the slot.
    pub expiration: String,
    /// LP token amount paid for the slot.
    pub price: Amount,
    /// Current 72-minute time interval within the 24-hour auction window.
    pub time_interval: u32,
}

/// An LP's vote on the pool trading fee.
#[derive(Debug, Clone, Deserialize)]
pub struct VoteSlot {
    /// Account that cast the vote.
    pub account: String,
    /// Proposed trading fee in units of 1/100,000.
    pub trading_fee: u32,
    /// Weight of this vote, proportional to the LP's token share.
    pub vote_weight: u32,
}

/// Full description of an AMM pool returned by `amm_info`.
#[derive(Debug, Clone, Deserialize)]
pub struct AmmDescription {
    /// AMM pool account address on the ledger.
    pub account: String,
    /// Balance of the first asset held by the pool.
    pub amount: Amount,
    /// Balance of the second asset held by the pool.
    pub amount2: Amount,
    /// Whether the first asset is currently frozen by its issuer.
    #[serde(default)]
    pub asset_frozen: Option<bool>,
    /// Whether the second asset is currently frozen by its issuer.
    #[serde(default)]
    pub asset2_frozen: Option<bool>,
    /// Active auction slot, if one has been purchased.
    pub auction_slot: Option<AuctionSlot>,
    /// Outstanding LP token supply for this pool.
    pub lp_token: Amount,
    /// Current trading fee in units of 1/100,000 (e.g. 500 = 0.5%).
    pub trading_fee: u32,
    /// LP fee votes currently in effect.
    pub vote_slots: Option<Vec<VoteSlot>>,
}

/// Response to an `amm_info` request.
#[derive(Debug, Clone, Deserialize)]
pub struct AmmInfoResponse {
    /// AMM pool state.
    pub amm: AmmDescription,
    /// Sequence number of the current open ledger (unvalidated results).
    pub ledger_current_index: Option<u32>,
    /// Hash of the ledger version used.
    pub ledger_hash: Option<String>,
    /// Sequence number of the ledger version used.
    pub ledger_index: Option<u32>,
    /// Whether the data comes from a validated ledger.
    pub validated: Option<bool>,
}
