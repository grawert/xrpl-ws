use crate::types::{Amount, ValidationError, validate_amount};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Builder for XRPL AMM withdraw transactions.
///
/// # Example
/// ```no_run
/// use xrpl::types::{Amount, builders::AMMWithdrawBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let asset = Amount::drops("100000000")?;
///     let asset2 = Amount::issued_currency("1000", "USD", "rIssuer")?;
///     let amm_withdraw = AMMWithdrawBuilder::new(asset, asset2)
///         .single_asset(Amount::drops("10000000")?)
///         .build()?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AMMWithdrawBuilder {
    asset: Amount,
    asset2: Amount,
    amount: Option<Amount>,
    amount2: Option<Amount>,
    e_price: Option<Amount>,
    lp_token_in: Option<Amount>,
    mode: Option<WithdrawMode>,
}

#[derive(Debug, Clone)]
enum WithdrawMode {
    LPToken,     // 0x00010000 - Double-asset, return specific LP tokens
    WithdrawAll, // 0x00020000 - Double-asset, return all LP tokens
    OneAssetWithdrawAll, // 0x00040000 - Single-asset, return all LP tokens
    SingleAsset, // 0x00080000 - Single-asset, specific amount
    TwoAsset,    // 0x00100000 - Double-asset, up to specified amounts
    OneAssetLPToken, // 0x00200000 - Single-asset, return specific LP tokens
    LimitLPToken, // 0x00400000 - Single-asset, with effective price limit
}

impl AMMWithdrawBuilder {
    /// Create a new AMM withdraw builder
    pub fn new(asset: Amount, asset2: Amount) -> Self {
        Self {
            asset,
            asset2,
            amount: None,
            amount2: None,
            e_price: None,
            lp_token_in: None,
            mode: None,
        }
    }

    /// Single-asset withdrawal of specific amount
    pub fn single_asset(mut self, amount: Amount) -> Self {
        self.amount = Some(amount);
        self.mode = Some(WithdrawMode::SingleAsset);
        self
    }

    /// Double-asset withdrawal up to specified amounts
    pub fn double_asset(mut self, amount: Amount, amount2: Amount) -> Self {
        self.amount = Some(amount);
        self.amount2 = Some(amount2);
        self.mode = Some(WithdrawMode::TwoAsset);
        self
    }

    /// Return specific amount of LP tokens for both assets
    pub fn lp_tokens_in(mut self, lp_tokens: Amount) -> Self {
        self.lp_token_in = Some(lp_tokens);
        self.mode = Some(WithdrawMode::LPToken);
        self
    }

    /// Withdraw all LP tokens for both assets
    pub fn withdraw_all(mut self) -> Self {
        self.mode = Some(WithdrawMode::WithdrawAll);
        self
    }

    /// Single-asset withdrawal returning all LP tokens
    pub fn single_asset_withdraw_all(mut self, min_amount: Amount) -> Self {
        self.amount = Some(min_amount);
        self.mode = Some(WithdrawMode::OneAssetWithdrawAll);
        self
    }

    /// Single-asset withdrawal with specific LP token amount
    pub fn single_asset_lp_tokens(
        mut self,
        amount: Amount,
        lp_tokens: Amount,
    ) -> Self {
        self.amount = Some(amount);
        self.lp_token_in = Some(lp_tokens);
        self.mode = Some(WithdrawMode::OneAssetLPToken);
        self
    }

    /// Single-asset withdrawal with effective price limit
    pub fn single_asset_with_limit(
        mut self,
        amount: Amount,
        min_price: Amount,
    ) -> Self {
        self.amount = Some(amount);
        self.e_price = Some(min_price);
        self.mode = Some(WithdrawMode::LimitLPToken);
        self
    }

    /// Build the withdraw transaction fields
    pub fn build(self) -> Result<AMMWithdrawFields, ValidationError> {
        validate_amount(&self.asset)?;
        validate_amount(&self.asset2)?;

        if let Some(ref amount) = self.amount {
            validate_amount(amount)?;
        }
        if let Some(ref amount2) = self.amount2 {
            validate_amount(amount2)?;
        }

        Ok(AMMWithdrawFields {
            asset: self.asset,
            asset2: self.asset2,
            amount: self.amount,
            amount2: self.amount2,
            e_price: self.e_price,
            lp_token_in: self.lp_token_in,
        })
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AMMWithdrawFields {
    #[serde(rename = "Asset")]
    pub asset: Amount,
    #[serde(rename = "Asset2")]
    pub asset2: Amount,
    #[serde(rename = "Amount")]
    pub amount: Option<Amount>,
    #[serde(rename = "Amount2")]
    pub amount2: Option<Amount>,
    #[serde(rename = "EPrice")]
    pub e_price: Option<Amount>,
    #[serde(rename = "LPTokenIn")]
    pub lp_token_in: Option<Amount>,
}
