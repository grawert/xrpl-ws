use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{validation::validate_amount, Amount, TransactionType};

/// Builder for XRPL Clawback transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{xrp, drops};
/// use xrpl::types::{Amount, builders::ClawbackBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let clawback = ClawbackBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         1,
///         drops!(10),
///         xrp!(100),
///     )
///     .with_holder("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string())
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct Clawback {
    pub amount: Amount,
    pub holder: Option<String>,
}

pub type ClawbackBuilder = TransactionBuilder<Clawback>;

impl ClawbackBuilder {
    pub fn new(
        account: String,
        sequence: u32,
        fee: Amount,
        amount: Amount,
    ) -> Self {
        Self::init(account, sequence, fee, Clawback { amount, holder: None })
    }

    pub fn with_holder(mut self, holder: String) -> Self {
        self.transaction_type.holder = Some(holder);
        self
    }
}

impl TransactionTypeBuilder for Clawback {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_amount(&self.amount)?;
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::Clawback {
            amount: self.amount,
            holder: self.holder,
        })
    }
}
