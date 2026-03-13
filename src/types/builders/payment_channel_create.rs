use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::{validate_address, validate_amount},
    Amount, TransactionType,
};

/// Builder for XRPL PaymentChannelCreate transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{xrp, drops};
/// use xrpl::types::{Amount, builders::PaymentChannelCreateBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let channel_create = PaymentChannelCreateBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
///         "aB44YfzW3309594CA14ECdeEKEH3SqnLkJ63CYRXkA3eZY".to_string(),
///         1,
///         drops!(10),
///         xrp!(100),
///         3600,
///     )
///     .with_destination_tag(12345)
///     .with_cancel_after(1234567890)
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct PaymentChannelCreate {
    pub amount: Amount,
    pub destination: String,
    pub public_key: String,
    pub settle_delay: u32,
    pub destination_tag: Option<u32>,
    pub cancel_after: Option<u32>,
}

pub type PaymentChannelCreateBuilder = TransactionBuilder<PaymentChannelCreate>;

impl PaymentChannelCreateBuilder {
    pub fn new(
        account: String,
        destination: String,
        public_key: String,
        sequence: u32,
        fee: Amount,
        amount: Amount,
        settle_delay: u32,
    ) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            PaymentChannelCreate {
                amount,
                destination,
                public_key,
                settle_delay,
                destination_tag: None,
                cancel_after: None,
            },
        )
    }

    pub fn with_destination_tag(mut self, tag: u32) -> Self {
        self.transaction_type.destination_tag = Some(tag);
        self
    }

    pub fn with_cancel_after(mut self, cancel_after: u32) -> Self {
        self.transaction_type.cancel_after = Some(cancel_after);
        self
    }
}

impl TransactionTypeBuilder for PaymentChannelCreate {
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
        Ok(TransactionType::PaymentChannelCreate {
            amount: self.amount,
            destination: self.destination,
            public_key: self.public_key,
            settle_delay: self.settle_delay,
            destination_tag: self.destination_tag,
            cancel_after: self.cancel_after,
        })
    }
}
