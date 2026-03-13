use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{Amount, TransactionType};

/// Builder for XRPL OracleDelete transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{drops};
/// use xrpl::types::{Amount, builders::OracleDeleteBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let oracle_delete = OracleDeleteBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         1,
///         drops!(10),
///         123,
///     )
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct OracleDelete {
    pub oracle_document_id: u32,
}

pub type OracleDeleteBuilder = TransactionBuilder<OracleDelete>;

impl OracleDeleteBuilder {
    pub fn new(
        account: String,
        sequence: u32,
        fee: Amount,
        oracle_document_id: u32,
    ) -> Self {
        Self::init(account, sequence, fee, OracleDelete { oracle_document_id })
    }
}

impl TransactionTypeBuilder for OracleDelete {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::OracleDelete {
            oracle_document_id: self.oracle_document_id,
        })
    }
}
