use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::{validate_address, validate_amount, validate_invoice_id},
    transactions::payment::CheckCreate,
    Amount, TransactionType,
};

/// Builder for XRPL CheckCreate transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, xrp, time::ripple_now, types::builders::CheckCreateBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = CheckCreateBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     xrp!(100),
/// )
/// .with_destination_tag(12345)
/// .with_expiration(ripple_now() + 86_400) // expires in 24 hours
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type CheckCreateBuilder = TransactionBuilder<CheckCreate>;

impl CheckCreateBuilder {
    /// Creates a new `CheckCreateBuilder` with the required destination and maximum payment amount.
    pub fn new(
        account: impl Into<String>,
        destination: impl Into<String>,
        send_max: impl Into<Amount>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            CheckCreate {
                destination: destination.into(),
                send_max: send_max.into(),
                destination_tag: None,
                expiration: None,
                invoice_id: None,
            },
        )
    }

    /// Sets the destination tag for routing within the recipient account.
    pub fn with_destination_tag(mut self, tag: u32) -> Self {
        self.transaction_type.destination_tag = Some(tag);
        self
    }

    /// Sets the Ripple-epoch time after which the check can no longer be cashed.
    pub fn with_expiration(mut self, expiration: u32) -> Self {
        self.transaction_type.expiration = Some(expiration);
        self
    }

    /// Sets the 64-character hex invoice ID for reconciliation.
    pub fn with_invoice_id(
        mut self,
        invoice_id: impl Into<String>,
    ) -> Result<Self, BuildError> {
        let id = invoice_id.into();
        validate_invoice_id(&id)?;
        self.transaction_type.invoice_id = Some(id);
        Ok(self)
    }
}

impl TransactionTypeBuilder for CheckCreate {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_address(&self.destination)?;
        validate_amount(&self.send_max)?;
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::CheckCreate(self))
    }
}
