use crate::types::{Amount, ValidationError, validate_amount};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Builder for XRPL AMM deposit transactions.
///
/// # Example
/// ```no_run
/// use xrpl::types::{Amount, builders::AMMDepositBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let asset = Amount::drops("100000000")?;
///     let asset2 = Amount::issued_currency("1000", "USD", "rIssuer")?;
///     let amm_deposit = AMMDepositBuilder::new(asset, asset2)
///         .double_asset(Amount::drops("10000000")?, Amount::issued_currency("100", "USD", "rIssuer")?)
///         .build()?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AMMDepositBuilder {
    asset: Amount,
    asset2: Amount,
    amount: Option<Amount>,
    amount2: Option<Amount>,
    e_price: Option<Amount>,
    lp_token_out: Option<Amount>,
    trading_fee: Option<u16>,
    mode: Option<DepositMode>,
}

#[derive(Debug, Clone)]
enum DepositMode {
    LPToken,         // 0x00010000 - Double-asset, receive specific LP tokens
    SingleAsset,     // 0x00080000 - Single-asset deposit
    TwoAsset,        // 0x00100000 - Double-asset, up to specified amounts
    OneAssetLPToken, // 0x00200000 - Single-asset, receive specific LP tokens
    LimitLPToken,    // 0x00400000 - Single-asset, with price limit
    TwoAssetIfEmpty, // 0x00800000 - Empty AMM special case
}

impl AMMDepositBuilder {
    /// Create a new AMM deposit builder
    pub fn new(asset: Amount, asset2: Amount) -> Self {
        Self {
            asset,
            asset2,
            amount: None,
            amount2: None,
            e_price: None,
            lp_token_out: None,
            trading_fee: None,
            mode: None,
        }
    }

    /// Single-asset deposit with specified amount
    pub fn single_asset(mut self, amount: Amount) -> Self {
        self.amount = Some(amount);
        self.mode = Some(DepositMode::SingleAsset);
        self
    }

    /// Double-asset deposit with specified amounts
    pub fn double_asset(mut self, amount: Amount, amount2: Amount) -> Self {
        self.amount = Some(amount);
        self.amount2 = Some(amount2);
        self.mode = Some(DepositMode::TwoAsset);
        self
    }

    /// Deposit to receive specific amount of LP tokens
    pub fn lp_tokens_out(mut self, lp_tokens: Amount) -> Self {
        self.lp_token_out = Some(lp_tokens);
        self.mode = Some(DepositMode::LPToken);
        self
    }

    /// Single-asset deposit to receive specific LP tokens
    pub fn single_asset_lp_tokens(
        mut self,
        amount: Amount,
        lp_tokens: Amount,
    ) -> Self {
        self.amount = Some(amount);
        self.lp_token_out = Some(lp_tokens);
        self.mode = Some(DepositMode::OneAssetLPToken);
        self
    }

    /// Single-asset deposit with price limit
    pub fn single_asset_with_limit(
        mut self,
        amount: Amount,
        max_price: Amount,
    ) -> Self {
        self.amount = Some(amount);
        self.e_price = Some(max_price);
        self.mode = Some(DepositMode::LimitLPToken);
        self
    }

    /// Special double-asset deposit for empty AMM
    pub fn empty_amm_deposit(
        mut self,
        amount: Amount,
        amount2: Amount,
    ) -> Self {
        self.amount = Some(amount);
        self.amount2 = Some(amount2);
        self.mode = Some(DepositMode::TwoAssetIfEmpty);
        self
    }

    /// Add a trading fee vote
    pub fn with_trading_fee_vote(mut self, fee: u16) -> Self {
        self.trading_fee = Some(fee);
        self
    }

    /// Build the deposit transaction fields
    pub fn build(self) -> Result<AMMDepositFields, ValidationError> {
        validate_amount(&self.asset)?;
        validate_amount(&self.asset2)?;

        if let Some(ref amount) = self.amount {
            validate_amount(amount)?;
        }
        if let Some(ref amount2) = self.amount2 {
            validate_amount(amount2)?;
        }

        if let Some(fee) = self.trading_fee {
            if fee > 1000 {
                return Err(ValidationError::InvalidAmount(
                    "Trading fee cannot exceed 1000 (1%)".into(),
                ));
            }
        }

        Ok(AMMDepositFields {
            asset: self.asset,
            asset2: self.asset2,
            amount: self.amount,
            amount2: self.amount2,
            e_price: self.e_price,
            lp_token_out: self.lp_token_out,
            trading_fee: self.trading_fee,
        })
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AMMDepositFields {
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
    #[serde(rename = "LPTokenOut")]
    pub lp_token_out: Option<Amount>,
    #[serde(rename = "TradingFee")]
    pub trading_fee: Option<u16>,
}
