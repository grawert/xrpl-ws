use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{Amount, TransactionType};

/// Builder for XRPL DIDDelete transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{drops};
/// use xrpl::types::{Amount, builders::DIDDeleteBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let did_delete = DIDDeleteBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         1,
///         drops!(10),
///     )
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct DIDDelete {
    // This transaction only uses common fields
}

pub type DIDDeleteBuilder = TransactionBuilder<DIDDelete>;

impl DIDDeleteBuilder {
    pub fn new(account: String, sequence: u32, fee: Amount) -> Self {
        Self::init(account, sequence, fee, DIDDelete {})
    }
}

impl TransactionTypeBuilder for DIDDelete {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::DIDDelete {})
    }
}
