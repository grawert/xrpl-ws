use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::{validate_address, validate_amount},
    transactions::escrow::EscrowCreate,
    Amount, TransactionType,
};

/// Builder for XRPL EscrowCreate transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # use xrpl::types::Amount;
/// use xrpl::{Client, xrp, types::builders::EscrowCreateBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = EscrowCreateBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     xrp!(10),
/// )
/// .with_destination_tag(12345)
/// .with_finish_after(960000000)
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type EscrowCreateBuilder = TransactionBuilder<EscrowCreate>;

impl EscrowCreateBuilder {
    /// Creates a new `EscrowCreateBuilder` with the required destination and lock amount.
    pub fn new(
        account: impl Into<String>,
        destination: impl Into<String>,
        amount: impl Into<Amount>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            EscrowCreate {
                amount: amount.into(),
                destination: destination.into(),
                cancel_after: None,
                finish_after: None,
                condition: None,
                destination_tag: None,
            },
        )
    }

    /// Sets the Ripple-epoch time after which the escrow can be cancelled.
    pub fn with_cancel_after(mut self, cancel_after: u32) -> Self {
        self.transaction_type.cancel_after = Some(cancel_after);
        self
    }

    /// Sets the Ripple-epoch time after which the escrow can be finished.
    pub fn with_finish_after(mut self, finish_after: u32) -> Self {
        self.transaction_type.finish_after = Some(finish_after);
        self
    }

    /// Sets the hex-encoded PREIMAGE-SHA-256 crypto-condition that must be fulfilled to release funds.
    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.transaction_type.condition = Some(condition.into());
        self
    }

    /// Sets the destination tag for routing within the recipient account.
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
        Ok(TransactionType::EscrowCreate(self))
    }
}
