use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{Amount, TransactionType};

/// Builder for XRPL TicketCreate transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{drops};
/// use xrpl::types::{Amount, builders::TicketCreateBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let ticket_create = TicketCreateBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         1,
///         drops!(10),
///         10,
///     )
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct TicketCreate {
    pub ticket_count: u32,
}

pub type TicketCreateBuilder = TransactionBuilder<TicketCreate>;

impl TicketCreateBuilder {
    pub fn new(
        account: String,
        sequence: u32,
        fee: Amount,
        ticket_count: u32,
    ) -> Self {
        Self::init(account, sequence, fee, TicketCreate { ticket_count })
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
        Ok(TransactionType::TicketCreate { ticket_count: self.ticket_count })
    }
}
