use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{transactions::payment::CheckCancel, Amount, TransactionType};

/// Builder for XRPL CheckCancel transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::CheckCancelBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = CheckCancelBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "838766BA2B995C00744175F69A1B11E32C3DBC40E64801A4056FCBD657F57334",
/// )
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type CheckCancelBuilder = TransactionBuilder<CheckCancel>;

impl CheckCancelBuilder {
    /// Creates a new `CheckCancelBuilder` targeting the given check ID.
    pub fn new(
        account: impl Into<String>,
        check_id: impl Into<String>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            CheckCancel { check_id: check_id.into() },
        )
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
        Ok(TransactionType::CheckCancel(self))
    }
}
