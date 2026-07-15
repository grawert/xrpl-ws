use crate::types::{
    Amount, Asset,
    builders::{BuildError, TransactionBuilder, TransactionTypeBuilder},
    transactions::{TransactionType, amm::AMMClawback},
    validation::{validate_address, validate_amount},
};

/// Builder for XRPL AMM clawback transactions.
///
/// # Example
/// ```rust
/// use xrpl::types::{Asset, Amount, builders::AMMClawbackBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let issuer = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
///     let asset = Asset::token("USD", issuer)?;
///     let asset2 = Asset::xrp();
///     let holder = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";
///     let amm_clawback = AMMClawbackBuilder::new(issuer, asset, asset2, holder)
///         .with_amount(Amount::issued_currency("50", "USD", issuer)?)
///         .build()?;
///     Ok(())
/// }
/// ```
pub type AMMClawbackBuilder = TransactionBuilder<AMMClawback>;

impl AMMClawbackBuilder {
    /// Creates a new AMM clawback builder
    pub fn new(
        account: impl AsRef<str>,
        asset: impl Into<Asset>,
        asset2: impl Into<Asset>,
        holder: impl AsRef<str>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            AMMClawback {
                asset: asset.into(),
                asset2: asset2.into(),
                amount: None,
                holder: holder.as_ref().to_string(),
            },
        )
    }

    /// Sets the maximum amount to claw back
    pub fn with_amount(mut self, amount: impl Into<Amount>) -> Self {
        self.transaction_type.amount = Some(amount.into());
        self
    }
}

impl TransactionTypeBuilder for AMMClawback {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(ref amount) = self.amount {
            validate_amount(amount)?;
        }
        validate_address(&self.holder)?;
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::AMMClawback(self))
    }
}
