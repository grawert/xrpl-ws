use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::validate_address, transactions::escrow::EscrowCancel, Amount,
    TransactionType,
};

/// Builder for XRPL EscrowCancel transactions.
///
/// # Example
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::EscrowCancelBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = EscrowCancelBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     123,
/// )
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type EscrowCancelBuilder = TransactionBuilder<EscrowCancel>;

impl EscrowCancelBuilder {
    /// Creates a new `EscrowCancelBuilder` targeting the specified escrow.
    pub fn new(
        account: impl AsRef<str>,
        owner: impl AsRef<str>,
        offer_sequence: u32,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            EscrowCancel { owner: owner.as_ref().to_string(), offer_sequence },
        )
    }
}

impl TransactionTypeBuilder for EscrowCancel {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_address(&self.owner)?;
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::EscrowCancel(self))
    }
}
