use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::validate_amount,
    transactions::payment_channel::PaymentChannelFund, Amount, TransactionType,
};

/// Builder for XRPL PaymentChannelFund transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # use xrpl::types::Amount;
/// use xrpl::{Client, xrp, types::builders::PaymentChannelFundBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = PaymentChannelFundBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "5DB01B7FFED6B67E6B0414DED11E051D2EE2B7619CE0EAA6286D67A3A4D5BDB3",
///     xrp!(5),
/// )
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type PaymentChannelFundBuilder = TransactionBuilder<PaymentChannelFund>;

impl PaymentChannelFundBuilder {
    /// Creates a new `PaymentChannelFundBuilder` for the specified channel and additional amount.
    pub fn new(
        account: impl AsRef<str>,
        channel: impl AsRef<str>,
        amount: impl Into<Amount>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            PaymentChannelFund {
                channel: channel.as_ref().to_string(),
                amount: amount.into(),
                expiration: None,
            },
        )
    }

    /// Sets the new Ripple-epoch expiration time for the channel.
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
        Ok(TransactionType::PaymentChannelFund(self))
    }
}
