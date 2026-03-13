use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{Amount, TransactionType};

/// Builder for XRPL OfferCancel transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{drops};
/// use xrpl::types::{Amount, builders::OfferCancelBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let offer_cancel = OfferCancelBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         1,
///         drops!(10),
///         123,
///     )
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct OfferCancel {
    pub offer_sequence: u32,
}

pub type OfferCancelBuilder = TransactionBuilder<OfferCancel>;

impl OfferCancelBuilder {
    pub fn new(
        account: String,
        sequence: u32,
        fee: Amount,
        offer_sequence: u32,
    ) -> Self {
        Self::init(account, sequence, fee, OfferCancel { offer_sequence })
    }
}

impl TransactionTypeBuilder for OfferCancel {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::OfferCancel { offer_sequence: self.offer_sequence })
    }
}
