use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{transactions::account::TicketCreate, Amount, TransactionType};

/// Builder for XRPL TicketCreate transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::TicketCreateBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = TicketCreateBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", 10)
///     .fill(&client)
///     .await?
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub type TicketCreateBuilder = TransactionBuilder<TicketCreate>;

impl TicketCreateBuilder {
    /// Creates a new `TicketCreateBuilder` for the given number of tickets.
    pub fn new(account: impl AsRef<str>, ticket_count: u32) -> Self {
        Self::init(account, 0, Amount::default(), TicketCreate { ticket_count })
    }
}

impl TransactionTypeBuilder for TicketCreate {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::TicketCreate(self))
    }
}
