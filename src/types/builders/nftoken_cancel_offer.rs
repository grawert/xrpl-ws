use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{transactions::nft::NFTokenCancelOffer, Amount, TransactionType};

/// Builder for XRPL NFTokenCancelOffer transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::NFTokenCancelOfferBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = NFTokenCancelOfferBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     vec!["A44D2B1668F8B2C634B35AD539E66D9A5A90F2B8A6DBE2A66B0F77892E5A4A3D"],
/// )
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type NFTokenCancelOfferBuilder = TransactionBuilder<NFTokenCancelOffer>;

impl NFTokenCancelOfferBuilder {
    /// Creates a new `NFTokenCancelOfferBuilder` with the initial list of offer IDs to cancel.
    pub fn new(
        account: impl AsRef<str>,
        nftoken_offers: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            NFTokenCancelOffer {
                nftoken_offers: nftoken_offers
                    .into_iter()
                    .map(|s| s.as_ref().to_string())
                    .collect(),
            },
        )
    }

    /// Appends an additional offer ID to the cancellation list.
    pub fn add_offer(mut self, offer: impl AsRef<str>) -> Self {
        self.transaction_type.nftoken_offers.push(offer.as_ref().to_string());
        self
    }
}

impl TransactionTypeBuilder for NFTokenCancelOffer {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::NFTokenCancelOffer(self))
    }
}
