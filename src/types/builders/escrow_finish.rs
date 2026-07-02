use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::validate_address, transactions::escrow::EscrowFinish, Amount,
    TransactionType,
};

/// Builder for XRPL EscrowFinish transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::EscrowFinishBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = EscrowFinishBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     123,
/// )
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type EscrowFinishBuilder = TransactionBuilder<EscrowFinish>;

impl EscrowFinishBuilder {
    /// Creates a new `EscrowFinishBuilder` targeting the specified escrow.
    pub fn new(
        account: impl AsRef<str>,
        owner: impl AsRef<str>,
        offer_sequence: u32,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            EscrowFinish {
                owner: owner.as_ref().to_string(),
                offer_sequence,
                condition: None,
                fulfillment: None,
            },
        )
    }

    /// Sets the hex-encoded PREIMAGE-SHA-256 condition originally placed on the escrow.
    pub fn with_condition(mut self, condition: impl AsRef<str>) -> Self {
        self.transaction_type.condition = Some(condition.as_ref().to_string());
        self
    }

    /// Sets the hex-encoded fulfillment that satisfies the escrow's condition.
    pub fn with_fulfillment(mut self, fulfillment: impl AsRef<str>) -> Self {
        self.transaction_type.fulfillment =
            Some(fulfillment.as_ref().to_string());
        self
    }
}

impl TransactionTypeBuilder for EscrowFinish {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_address(&self.owner)?;
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::EscrowFinish(self))
    }
}
