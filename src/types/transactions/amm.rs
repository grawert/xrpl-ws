use std::ops::BitOr;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Transaction flags for [`AMMDeposit`].
///
/// Exactly one mode flag must be set. Combine with common flags using `|`.
///
/// ```rust
/// use xrpl::types::AMMDepositFlags as Flags;
///
/// let flags = Flags::SINGLE_ASSET; // deposit one asset, receive LP tokens
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AMMDepositFlags(pub u32);

impl AMMDepositFlags {
    /// Deposit both assets proportionally and receive LP tokens.
    pub const LP_TOKEN: Self = Self(0x00010000);
    /// Deposit a single asset and receive LP tokens.
    pub const SINGLE_ASSET: Self = Self(0x00080000);
    /// Deposit both assets at the current pool ratio.
    pub const TWO_ASSET: Self = Self(0x00100000);
    /// Deposit a single asset to obtain a specified LP token amount.
    pub const ONE_ASSET_LP_TOKEN: Self = Self(0x00200000);
    /// Deposit a single asset with a price limit per LP token.
    pub const LIMIT_LP_TOKEN: Self = Self(0x00400000);

    /// Returns `true` if the given flag is set in this bitmask.
    pub fn has(self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }
}

impl BitOr for AMMDepositFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl From<u32> for AMMDepositFlags {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<AMMDepositFlags> for u32 {
    fn from(f: AMMDepositFlags) -> u32 {
        f.0
    }
}

/// Transaction flags for [`AMMWithdraw`].
///
/// Exactly one mode flag must be set. Combine with common flags using `|`.
///
/// ```rust
/// use xrpl::types::AMMWithdrawFlags as Flags;
///
/// let flags = Flags::LP_TOKEN; // redeem LP tokens for both assets
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AMMWithdrawFlags(pub u32);

impl AMMWithdrawFlags {
    /// Redeem LP tokens for a proportional share of both assets.
    pub const LP_TOKEN: Self = Self(0x00010000);
    /// Withdraw all liquidity, burning all LP tokens held.
    pub const WITHDRAW_ALL: Self = Self(0x00020000);
    /// Withdraw all of one asset, paying LP tokens.
    pub const ONE_ASSET_WITHDRAW_ALL: Self = Self(0x00040000);
    /// Withdraw a specific amount of one asset.
    pub const SINGLE_ASSET: Self = Self(0x00080000);
    /// Withdraw both assets at a given ratio.
    pub const TWO_ASSET: Self = Self(0x00100000);
    /// Withdraw one asset and pay a specified LP token amount.
    pub const ONE_ASSET_LP_TOKEN: Self = Self(0x00200000);
    /// Withdraw one asset with a minimum LP token exchange rate.
    pub const LIMIT_LP_TOKEN: Self = Self(0x00400000);

    /// Returns `true` if the given flag is set in this bitmask.
    pub fn has(self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }
}

impl BitOr for AMMWithdrawFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl From<u32> for AMMWithdrawFlags {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<AMMWithdrawFlags> for u32 {
    fn from(f: AMMWithdrawFlags) -> u32 {
        f.0
    }
}

use crate::types::{Amount, Asset, AuthAccountWrapper};

/// Bids on the AMM auction slot to receive a discounted trading fee for a limited time.
///
/// The winning bidder pays LP tokens and can authorize up to four additional accounts
/// to also receive the discounted fee.
///
/// ```rust
/// use xrpl::types::{Asset, transactions::amm::AMMBid};
/// let tx = AMMBid {
///     asset: Asset::xrp(),
///     asset2: Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap(),
///     bid_min: None,
///     bid_max: None,
///     auth_accounts: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AMMBid {
    /// First asset of the AMM pool.
    #[serde(rename = "Asset")]
    pub asset: Asset,
    /// Second asset of the AMM pool.
    #[serde(rename = "Asset2")]
    pub asset2: Asset,
    /// Minimum LP token amount the bidder is willing to pay.
    #[serde(rename = "BidMin")]
    pub bid_min: Option<Amount>,
    /// Maximum LP token amount the bidder is willing to pay.
    #[serde(rename = "BidMax")]
    pub bid_max: Option<Amount>,
    /// Accounts that also receive the discounted fee while the slot is held (up to 4).
    #[serde(rename = "AuthAccounts")]
    pub auth_accounts: Option<Vec<AuthAccountWrapper>>,
}

/// Issuer clawback of tokens held inside an AMM pool.
///
/// Available only when the token issuance has clawback enabled. Removes the specified
/// holder's share of the issuer's token from the AMM pool.
///
/// ```rust
/// use xrpl::types::{Amount, Asset, transactions::amm::AMMClawback};
/// let tx = AMMClawback {
///     asset: Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap(),
///     asset2: Asset::xrp(),
///     amount: None, // claw back all if omitted
///     holder: "rHolderAccount".to_string(),
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AMMClawback {
    /// The issuer's token asset in the pool.
    #[serde(rename = "Asset")]
    pub asset: Asset,
    /// The paired asset in the pool.
    #[serde(rename = "Asset2")]
    pub asset2: Asset,
    /// Maximum amount to claw back; omit to claw back the full balance.
    #[serde(rename = "Amount")]
    pub amount: Option<Amount>,
    /// The account holding the tokens to be clawed back.
    #[serde(rename = "Holder")]
    pub holder: String,
}

/// Initializes a new Automated Market Maker (AMM) pool with two assets and a trading fee.
///
/// The submitting account provides the initial liquidity for both assets and receives
/// LP tokens representing its share of the pool.
///
/// ```rust
/// use xrpl::types::{Amount, transactions::amm::AMMCreate};
/// let tx = AMMCreate {
///     amount: Amount::Xrpl("50000000".to_string()),
///     amount2: Amount::IssuedCurrency {
///         value: "500".to_string(),
///         currency: "USD".to_string(),
///         issuer: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///     },
///     trading_fee: 500, // 0.5%
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AMMCreate {
    /// Initial deposit of the first asset.
    #[serde(rename = "Amount")]
    pub amount: Amount,
    /// Initial deposit of the second asset.
    #[serde(rename = "Amount2")]
    pub amount2: Amount,
    /// Trading fee in units of 1/100,000 of a percent (0-1000, i.e. 0%-1%).
    #[serde(rename = "TradingFee")]
    pub trading_fee: u16,
}

/// Removes an AMM pool that has been reduced to an empty or dust state.
///
/// Any account can submit this transaction to clean up a pool with no remaining
/// liquidity and claim the reserve that was locked by the pool object.
///
/// ```rust
/// use xrpl::types::{Asset, transactions::amm::AMMDelete};
/// let tx = AMMDelete {
///     asset: Asset::xrp(),
///     asset2: Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap(),
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AMMDelete {
    /// First asset of the pool to remove.
    #[serde(rename = "Asset")]
    pub asset: Asset,
    /// Second asset of the pool to remove.
    #[serde(rename = "Asset2")]
    pub asset2: Asset,
}

/// Adds liquidity to an existing AMM pool and receives LP tokens in return.
///
/// Supports several deposit modes selected by combining the optional fields and
/// the transaction flags (e.g. single-asset, double-asset, or LP-token-targeted).
///
/// ```rust
/// use xrpl::types::{Amount, Asset, transactions::amm::AMMDeposit};
/// let tx = AMMDeposit {
///     asset: Asset::xrp(),
///     asset2: Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap(),
///     amount: Some(Amount::Xrpl("10000000".to_string())),
///     amount2: None,
///     e_price: None,
///     lp_token_out: None,
///     trading_fee: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AMMDeposit {
    /// First asset of the pool.
    #[serde(rename = "Asset")]
    pub asset: Asset,
    /// Second asset of the pool.
    #[serde(rename = "Asset2")]
    pub asset2: Asset,
    /// Maximum amount of the first asset to deposit.
    #[serde(rename = "Amount")]
    pub amount: Option<Amount>,
    /// Maximum amount of the second asset to deposit.
    #[serde(rename = "Amount2")]
    pub amount2: Option<Amount>,
    /// Effective price limit per LP token when using the `LimitLPToken` mode.
    #[serde(rename = "EPrice")]
    pub e_price: Option<Amount>,
    /// Exact number of LP tokens the depositor wants to receive.
    #[serde(rename = "LPTokenOut")]
    pub lp_token_out: Option<Amount>,
    /// Trading fee vote to submit alongside the deposit (0-1000).
    #[serde(rename = "TradingFee")]
    pub trading_fee: Option<u16>,
}

/// Votes on the trading fee for an AMM pool.
///
/// LP token holders can vote to change the pool's trading fee. The effective fee is
/// the LP-token-weighted average of all active votes.
///
/// ```rust
/// use xrpl::types::{Asset, transactions::amm::AMMVote};
/// let tx = AMMVote {
///     asset: Asset::xrp(),
///     asset2: Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap(),
///     trading_fee: 500, // vote for 0.5%
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AMMVote {
    /// First asset of the pool.
    #[serde(rename = "Asset")]
    pub asset: Asset,
    /// Second asset of the pool.
    #[serde(rename = "Asset2")]
    pub asset2: Asset,
    /// Proposed trading fee in units of 1/100,000 of a percent (0-1000).
    #[serde(rename = "TradingFee")]
    pub trading_fee: u16,
}

/// Redeems LP tokens and withdraws assets from an AMM pool.
///
/// Supports several withdrawal modes selected by combining the optional fields and
/// transaction flags (e.g. single-asset, double-asset, or full withdrawal).
///
/// ```rust
/// use xrpl::types::{Amount, Asset, transactions::amm::AMMWithdraw};
/// let tx = AMMWithdraw {
///     asset: Asset::xrp(),
///     asset2: Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap(),
///     lp_token_in: Some(Amount::IssuedCurrency {
///         value: "100".to_string(),
///         currency: "03930D02208264E2E40EC1B0C09E4DB96EE197B1".to_string(),
///         issuer: "rAMMPool".to_string(),
///     }),
///     amount: None,
///     amount2: None,
///     e_price: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AMMWithdraw {
    /// First asset of the pool.
    #[serde(rename = "Asset")]
    pub asset: Asset,
    /// Second asset of the pool.
    #[serde(rename = "Asset2")]
    pub asset2: Asset,
    /// Minimum amount of the first asset to receive.
    #[serde(rename = "Amount")]
    pub amount: Option<Amount>,
    /// Minimum amount of the second asset to receive.
    #[serde(rename = "Amount2")]
    pub amount2: Option<Amount>,
    /// Effective price limit per LP token when using the `LimitLPToken` mode.
    #[serde(rename = "EPrice")]
    pub e_price: Option<Amount>,
    /// Exact number of LP tokens to redeem.
    #[serde(rename = "LPTokenIn")]
    pub lp_token_in: Option<Amount>,
}
