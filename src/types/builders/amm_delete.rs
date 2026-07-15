use crate::types::{
    Amount, Asset,
    builders::{BuildError, TransactionBuilder, TransactionTypeBuilder},
    transactions::{TransactionType, amm::AMMDelete},
    validation::ValidationError,
};

/// Builder for XRPL AMM delete transactions.
///
/// # Example
/// ```rust
/// use xrpl::types::{Asset, builders::AMMDeleteBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let asset = Asset::xrp();
///     let asset2 = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
///     let amm_delete = AMMDeleteBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", asset, asset2)
///         .build()?;
///     Ok(())
/// }
/// ```
pub type AMMDeleteBuilder = TransactionBuilder<AMMDelete>;

impl AMMDeleteBuilder {
    /// Creates a new AMM delete builder
    pub fn new(
        account: impl AsRef<str>,
        asset: impl Into<Asset>,
        asset2: impl Into<Asset>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            AMMDelete { asset: asset.into(), asset2: asset2.into() },
        )
    }
}

impl TransactionTypeBuilder for AMMDelete {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if self.asset == self.asset2 {
            return Err(ValidationError::InvalidAmount(
                "Asset and Asset2 must be different".into(),
            )
            .into());
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::AMMDelete(self))
    }
}
