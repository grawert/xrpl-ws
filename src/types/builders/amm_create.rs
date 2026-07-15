use crate::types::{
    Amount,
    builders::{BuildError, TransactionBuilder, TransactionTypeBuilder},
    transactions::{TransactionType, amm::AMMCreate},
    validation::{validate_amount, ValidationError},
};

/// Builder for XRPL AMM create transactions.
///
/// # Example
/// ```rust
/// use xrpl::types::{Amount, builders::AMMCreateBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let amount = Amount::drops("50000000")?;
///     let amount2 = Amount::issued_currency("500", "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
///     let amm_create = AMMCreateBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", amount, amount2, 500)
///         .build()?;
///     Ok(())
/// }
/// ```
pub type AMMCreateBuilder = TransactionBuilder<AMMCreate>;

impl AMMCreateBuilder {
    /// Creates a new AMMCreate builder
    pub fn new(
        account: impl AsRef<str>,
        amount: impl Into<Amount>,
        amount2: impl Into<Amount>,
        trading_fee: u16,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            AMMCreate {
                amount: amount.into(),
                amount2: amount2.into(),
                trading_fee,
            },
        )
    }
}

impl TransactionTypeBuilder for AMMCreate {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_amount(&self.amount)?;
        validate_amount(&self.amount2)?;

        if self.trading_fee > 1000 {
            return Err(ValidationError::InvalidAmount(
                "Trading fee cannot exceed 1000 (1%)".into(),
            )
            .into());
        }

        if self.amount == self.amount2 {
            return Err(ValidationError::InvalidAmount(
                "Assets must be different".into(),
            )
            .into());
        }

        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::AMMCreate(self))
    }
}
