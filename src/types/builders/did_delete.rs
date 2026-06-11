use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{transactions::did::DIDDelete, Amount, TransactionType};

/// Builder for XRPL DIDDelete transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::DIDDeleteBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = DIDDeleteBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
///     .fill(&client)
///     .await?
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub type DIDDeleteBuilder = TransactionBuilder<DIDDelete>;

impl DIDDeleteBuilder {
    /// Creates a new `DIDDeleteBuilder`.
    pub fn new(account: impl AsRef<str>) -> Self {
        Self::init(account, 0, Amount::default(), DIDDelete {})
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
        Ok(TransactionType::DIDDelete(self))
    }
}
