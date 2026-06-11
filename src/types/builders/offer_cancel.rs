use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{transactions::offer::OfferCancel, Amount, TransactionType};

/// Builder for XRPL OfferCancel transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::OfferCancelBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = OfferCancelBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", 123)
///     .fill(&client)
///     .await?
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub type OfferCancelBuilder = TransactionBuilder<OfferCancel>;

impl OfferCancelBuilder {
    /// Creates a new `OfferCancelBuilder` targeting the offer at the given sequence number.
    pub fn new(account: impl AsRef<str>, offer_sequence: u32) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            OfferCancel { offer_sequence },
        )
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
        Ok(TransactionType::OfferCancel(self))
    }
}
