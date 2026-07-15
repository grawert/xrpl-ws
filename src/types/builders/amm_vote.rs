use crate::types::{
    Amount, Asset,
    builders::{BuildError, TransactionBuilder, TransactionTypeBuilder},
    transactions::{TransactionType, amm::AMMVote},
    validation::ValidationError,
};

/// Builder for XRPL AMM vote transactions.
///
/// # Example
/// ```rust
/// use xrpl::types::{Asset, builders::AMMVoteBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let asset = Asset::xrp();
///     let asset2 = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
///     let amm_vote = AMMVoteBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", asset, asset2, 500)
///         .build()?;
///     Ok(())
/// }
/// ```
pub type AMMVoteBuilder = TransactionBuilder<AMMVote>;

impl AMMVoteBuilder {
    /// Creates a new AMM vote builder
    pub fn new(
        account: impl AsRef<str>,
        asset: impl Into<Asset>,
        asset2: impl Into<Asset>,
        trading_fee: u16,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            AMMVote { asset: asset.into(), asset2: asset2.into(), trading_fee },
        )
    }
}

impl TransactionTypeBuilder for AMMVote {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if self.trading_fee > 1000 {
            return Err(ValidationError::InvalidAmount(
                "Trading fee cannot exceed 1000 (1%)".into(),
            )
            .into());
        }
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
        Ok(TransactionType::AMMVote(self))
    }
}
