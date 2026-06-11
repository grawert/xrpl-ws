use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::{validate_address, validate_amount},
    transactions::payment_channel::PaymentChannelCreate,
    Amount, TransactionType,
};

/// Builder for XRPL PaymentChannelCreate transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # use xrpl::types::Amount;
/// use xrpl::{Client, xrp, types::builders::PaymentChannelCreateBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = PaymentChannelCreateBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "ED5E6F48B2B1E8C7D2C3F5A4B6E8D9F0A1C2D3E4F5A6B7C8D9E0F1A2B3C4D5E6F",
///     xrp!(10),
///     86400,
/// )
/// .with_destination_tag(12345)
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type PaymentChannelCreateBuilder = TransactionBuilder<PaymentChannelCreate>;

impl PaymentChannelCreateBuilder {
    /// Creates a new `PaymentChannelCreateBuilder` with the required fields.
    pub fn new(
        account: impl AsRef<str>,
        destination: impl AsRef<str>,
        public_key: impl AsRef<str>,
        amount: impl Into<Amount>,
        settle_delay: u32,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            PaymentChannelCreate {
                amount: amount.into(),
                destination: destination.as_ref().to_string(),
                public_key: public_key.as_ref().to_string(),
                settle_delay,
                destination_tag: None,
                cancel_after: None,
            },
        )
    }

    /// Sets the destination tag for routing within the recipient account.
    pub fn with_destination_tag(mut self, tag: u32) -> Self {
        self.transaction_type.destination_tag = Some(tag);
        self
    }

    /// Sets the Ripple-epoch time after which the channel can be closed by anyone.
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
        Ok(TransactionType::PaymentChannelCreate(self))
    }
}
