use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{validation::validate_amount, Amount, TransactionType};

/// Builder for XRPL PaymentChannelFund transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{xrp, drops};
/// use xrpl::types::{Amount, builders::PaymentChannelFundBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let channel_fund = PaymentChannelFundBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         "5DB01B7FFED6B67E6B0414DED11E051D2EE2B7619CE0EAA6286D67A3A4D5BDB3".to_string(),
///         1,
///         drops!(10),
///         xrp!(100),
///     )
///     .with_expiration(1234567890)
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct PaymentChannelFund {
    pub channel: String,
    pub amount: Amount,
    pub expiration: Option<u32>,
}

pub type PaymentChannelFundBuilder = TransactionBuilder<PaymentChannelFund>;

impl PaymentChannelFundBuilder {
    pub fn new(
        account: String,
        channel: String,
        sequence: u32,
        fee: Amount,
        amount: Amount,
    ) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            PaymentChannelFund { channel, amount, expiration: None },
        )
    }

    pub fn with_expiration(mut self, expiration: u32) -> Self {
        self.transaction_type.expiration = Some(expiration);
        self
    }
}

impl TransactionTypeBuilder for PaymentChannelFund {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_amount(&self.amount)?;
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::PaymentChannelFund {
            channel: self.channel,
            amount: self.amount,
            expiration: self.expiration,
        })
    }
}
