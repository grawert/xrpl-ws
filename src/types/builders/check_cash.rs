use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{validation::validate_amount, Amount, TransactionType};

/// Builder for XRPL CheckCash transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{xrp, drops};
/// use xrpl::types::{Amount, builders::CheckCashBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let check_cash = CheckCashBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         "838766BA2B995C00744175F69A1B11E32C3DBC40E64801A4056FCBD657F57334".to_string(),
///         1,
///         drops!(10),
///     )
///     .with_amount(xrp!(100))
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct CheckCash {
    pub check_id: String,
    pub amount: Option<Amount>,
    pub deliver_min: Option<Amount>,
}

pub type CheckCashBuilder = TransactionBuilder<CheckCash>;

impl CheckCashBuilder {
    pub fn new(
        account: String,
        check_id: String,
        sequence: u32,
        fee: Amount,
    ) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            CheckCash { check_id, amount: None, deliver_min: None },
        )
    }

    pub fn with_amount(mut self, amount: Amount) -> Self {
        self.transaction_type.amount = Some(amount);
        self
    }

    pub fn with_deliver_min(mut self, deliver_min: Amount) -> Self {
        self.transaction_type.deliver_min = Some(deliver_min);
        self
    }
}

impl TransactionTypeBuilder for CheckCash {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(amount) = &self.amount {
            validate_amount(amount)?;
        }
        if let Some(deliver_min) = &self.deliver_min {
            validate_amount(deliver_min)?;
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::CheckCash {
            check_id: self.check_id,
            amount: self.amount,
            deliver_min: self.deliver_min,
        })
    }
}
