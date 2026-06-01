use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{transactions::oracle::OracleDelete, Amount, TransactionType};

/// Builder for XRPL OracleDelete transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::OracleDeleteBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = OracleDeleteBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", 123)
///     .fill(&client)
///     .await?
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub type OracleDeleteBuilder = TransactionBuilder<OracleDelete>;

impl OracleDeleteBuilder {
    /// Creates a new `OracleDeleteBuilder` targeting the given oracle document ID.
    pub fn new(account: impl Into<String>, oracle_document_id: u32) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            OracleDelete { oracle_document_id },
        )
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
        Ok(TransactionType::OracleDelete(self))
    }
}
