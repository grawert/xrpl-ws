use crate::types::{Amount, ValidationError, validate_amount};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Builder for XRPL AMM vote transactions.
///
/// # Example
/// ```no_run
/// use xrpl::types::{Amount, builders::AMMVoteBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let asset = Amount::drops("100000000")?;
///     let asset2 = Amount::issued_currency("1000", "USD", "rIssuer")?;
///     let amm_vote = AMMVoteBuilder::new(asset, asset2)
///         .trading_fee(500)
///         .build()?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AMMVoteBuilder {
    asset: Amount,
    asset2: Amount,
    trading_fee: u16,
}

impl AMMVoteBuilder {
    /// Create a new AMM vote builder
    pub fn new(asset: Amount, asset2: Amount) -> Self {
        Self { asset, asset2, trading_fee: 0 }
    }

    /// Set the trading fee to vote for (0-1000, representing 0-1%)
    pub fn trading_fee(mut self, fee: u16) -> Self {
        self.trading_fee = fee;
        self
    }

    /// Vote for a low trading fee (0.1%)
    pub fn low_fee(mut self) -> Self {
        self.trading_fee = 100;
        self
    }

    /// Vote for a medium trading fee (0.5%)
    pub fn medium_fee(mut self) -> Self {
        self.trading_fee = 500;
        self
    }

    /// Vote for a high trading fee (1.0%)
    pub fn high_fee(mut self) -> Self {
        self.trading_fee = 1000;
        self
    }

    /// Build the vote transaction fields
    pub fn build(self) -> Result<AMMVoteFields, ValidationError> {
        validate_amount(&self.asset)?;
        validate_amount(&self.asset2)?;

        if self.trading_fee > 1000 {
            return Err(ValidationError::InvalidAmount(
                "Trading fee cannot exceed 1000 (1%)".into(),
            ));
        }

        Ok(AMMVoteFields {
            asset: self.asset,
            asset2: self.asset2,
            trading_fee: self.trading_fee,
        })
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AMMVoteFields {
    #[serde(rename = "Asset")]
    pub asset: Amount,
    #[serde(rename = "Asset2")]
    pub asset2: Amount,
    #[serde(rename = "TradingFee")]
    pub trading_fee: u16,
}
