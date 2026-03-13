use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::{validate_address, validate_amount},
    Amount, TransactionType,
};

/// Builder for XRPL EscrowCreate transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{xrp, drops};
/// use xrpl::types::{Amount, builders::EscrowCreateBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let escrow_create = EscrowCreateBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
///         1,
///         drops!(10),
///         xrp!(100),
///     )
///     .with_destination_tag(12345)
///     .with_finish_after(1234567890)
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct EscrowCreate {
    pub amount: Amount,
    pub destination: String,
    pub cancel_after: Option<u32>,
    pub finish_after: Option<u32>,
    pub condition: Option<String>,
    pub destination_tag: Option<u32>,
}

pub type EscrowCreateBuilder = TransactionBuilder<EscrowCreate>;

impl EscrowCreateBuilder {
    pub fn new(
        account: String,
        destination: String,
        sequence: u32,
        fee: Amount,
        amount: Amount,
    ) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            EscrowCreate {
                amount,
                destination,
                cancel_after: None,
                finish_after: None,
                condition: None,
                destination_tag: None,
            },
        )
    }

    pub fn with_cancel_after(mut self, cancel_after: u32) -> Self {
        self.transaction_type.cancel_after = Some(cancel_after);
        self
    }

    pub fn with_finish_after(mut self, finish_after: u32) -> Self {
        self.transaction_type.finish_after = Some(finish_after);
        self
    }

    pub fn with_condition(mut self, condition: String) -> Self {
        self.transaction_type.condition = Some(condition);
        self
    }

    pub fn with_destination_tag(mut self, tag: u32) -> Self {
        self.transaction_type.destination_tag = Some(tag);
        self
    }
}

impl TransactionTypeBuilder for EscrowCreate {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_address(&self.destination)?;
        validate_amount(&self.amount)?;
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::EscrowCreate {
            amount: self.amount,
            destination: self.destination,
            cancel_after: self.cancel_after,
            finish_after: self.finish_after,
            condition: self.condition,
            destination_tag: self.destination_tag,
        })
    }
}
