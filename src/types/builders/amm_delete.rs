use crate::types::{Amount, ValidationError, validate_amount};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Builder for XRPL AMM delete transactions.
///
/// # Example
/// ```no_run
/// use xrpl::types::{Amount, builders::AMMDeleteBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let asset = Amount::drops("100000000")?;
///     let asset2 = Amount::issued_currency("1000", "USD", "rIssuer")?;
///     let amm_delete = AMMDeleteBuilder::new(asset, asset2)
///         .build()?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AMMDeleteBuilder {
    asset: Amount,
    asset2: Amount,
}

impl AMMDeleteBuilder {
    /// Create a new AMM delete builder
    pub fn new(asset: Amount, asset2: Amount) -> Self {
        Self { asset, asset2 }
    }

    /// Build the delete transaction fields
    pub fn build(self) -> Result<AMMDeleteFields, ValidationError> {
        validate_amount(&self.asset)?;
        validate_amount(&self.asset2)?;

        Ok(AMMDeleteFields { asset: self.asset, asset2: self.asset2 })
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AMMDeleteFields {
    #[serde(rename = "Asset")]
    pub asset: Amount,
    #[serde(rename = "Asset2")]
    pub asset2: Amount,
}
