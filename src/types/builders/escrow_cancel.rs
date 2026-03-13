use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{validation::validate_address, Amount, TransactionType};

/// Builder for XRPL EscrowCancel transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{drops};
/// use xrpl::types::{Amount, builders::EscrowCancelBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let escrow_cancel = EscrowCancelBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
///         1,
///         drops!(10),
///         123,
///     )
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct EscrowCancel {
    pub owner: String,
    pub offer_sequence: u32,
}

pub type EscrowCancelBuilder = TransactionBuilder<EscrowCancel>;

impl EscrowCancelBuilder {
    pub fn new(
        account: String,
        owner: String,
        sequence: u32,
        fee: Amount,
        offer_sequence: u32,
    ) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            EscrowCancel { owner, offer_sequence },
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
        Ok(TransactionType::EscrowCancel {
            owner: self.owner,
            offer_sequence: self.offer_sequence,
        })
    }
}
