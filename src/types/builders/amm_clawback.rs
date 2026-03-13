use crate::types::{Amount, ValidationError, validate_amount, validate_address};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Builder for XRPL AMM clawback transactions.
///
/// # Example
/// ```no_run
/// use xrpl::types::{Amount, builders::AMMClawbackBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let asset = Amount::drops("100000000")?;
///     let asset2 = Amount::issued_currency("1000", "USD", "rIssuer")?;
///     let holder = "rHolder123".to_string();
///     let amm_clawback = AMMClawbackBuilder::new(asset, asset2, holder)
///         .amount(Amount::drops("50000000")?)
///         .build()?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AMMClawbackBuilder {
    asset: Amount,
    asset2: Amount,
    amount: Option<Amount>,
    holder: String,
}

impl AMMClawbackBuilder {
    /// Create a new AMM clawback builder
    pub fn new(asset: Amount, asset2: Amount, holder: String) -> Self {
        Self { asset, asset2, amount: None, holder }
    }

    /// Set the maximum amount to claw back
    pub fn amount(mut self, amount: Amount) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Build the clawback transaction fields
    pub fn build(self) -> Result<AMMClawbackFields, ValidationError> {
        validate_amount(&self.asset)?;
        validate_amount(&self.asset2)?;
        validate_address(&self.holder)?;

        if let Some(ref amount) = self.amount {
            validate_amount(amount)?;
        }

        Ok(AMMClawbackFields {
            asset: self.asset,
            asset2: self.asset2,
            amount: self.amount,
            holder: self.holder,
        })
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AMMClawbackFields {
    #[serde(rename = "Asset")]
    pub asset: Amount,
    #[serde(rename = "Asset2")]
    pub asset2: Amount,
    #[serde(rename = "Amount")]
    pub amount: Option<Amount>,
    #[serde(rename = "Holder")]
    pub holder: String,
}
