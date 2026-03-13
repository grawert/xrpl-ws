use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{Amount, TransactionType};

/// Builder for XRPL CheckCancel transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{drops};
/// use xrpl::types::{Amount, builders::CheckCancelBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let check_cancel = CheckCancelBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         "838766BA2B995C00744175F69A1B11E32C3DBC40E64801A4056FCBD657F57334".to_string(),
///         1,
///         drops!(10),
///     )
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct CheckCancel {
    pub check_id: String,
}

pub type CheckCancelBuilder = TransactionBuilder<CheckCancel>;

impl CheckCancelBuilder {
    pub fn new(
        account: String,
        check_id: String,
        sequence: u32,
        fee: Amount,
    ) -> Self {
        Self::init(account, sequence, fee, CheckCancel { check_id })
    }
}

impl TransactionTypeBuilder for CheckCancel {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::CheckCancel { check_id: self.check_id })
    }
}
