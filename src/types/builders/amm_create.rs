use crate::types::{Amount, ValidationError, validate_amount};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Builder for XRPL AMM create transactions.
///
/// # Example
/// ```no_run
/// use xrpl::types::{Amount, builders::AMMCreateBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let amount = Amount::drops("50000000")?;
///     let amount2 = Amount::issued_currency("500", "USD", "rIssuer")?;
///     let amm_create = AMMCreateBuilder::new(amount, amount2)
///         .trading_fee(500)
///         .build()?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AMMCreateBuilder {
    amount: Amount,
    amount2: Amount,
    trading_fee: u16,
}

impl AMMCreateBuilder {
    /// Create a new AMMCreate builder with the two initial amounts
    pub fn new(amount: Amount, amount2: Amount) -> Self {
        Self {
            amount,
            amount2,
            trading_fee: 0, // Default to 0% fee
        }
    }

    /// Set the trading fee (0-1000, representing 0-1%)
    pub fn trading_fee(mut self, fee: u16) -> Self {
        self.trading_fee = fee;
        self
    }

    /// Set a low trading fee (0.1%)
    pub fn low_fee(mut self) -> Self {
        self.trading_fee = 100;
        self
    }

    /// Set a medium trading fee (0.5%)
    pub fn medium_fee(mut self) -> Self {
        self.trading_fee = 500;
        self
    }

    /// Set a high trading fee (1.0%)
    pub fn high_fee(mut self) -> Self {
        self.trading_fee = 1000;
        self
    }

    /// Validate and build the AMMCreate transaction fields
    pub fn build(self) -> Result<AMMCreateFields, ValidationError> {
        // Validate amounts
        validate_amount(&self.amount)?;
        validate_amount(&self.amount2)?;

        // Validate trading fee range
        if self.trading_fee > 1000 {
            return Err(ValidationError::InvalidAmount(
                "Trading fee cannot exceed 1000 (1%)".into(),
            ));
        }

        // Ensure assets are different
        if self.amount == self.amount2 {
            return Err(ValidationError::InvalidAmount(
                "Assets must be different".into(),
            ));
        }

        Ok(AMMCreateFields {
            amount: self.amount,
            amount2: self.amount2,
            trading_fee: self.trading_fee,
        })
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AMMCreateFields {
    #[serde(rename = "Amount")]
    pub amount: Amount,
    #[serde(rename = "Amount2")]
    pub amount2: Amount,
    #[serde(rename = "TradingFee")]
    pub trading_fee: u16,
}
