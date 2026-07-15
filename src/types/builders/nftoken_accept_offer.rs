use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::validate_amount, transactions::nft::NFTokenAcceptOffer, Amount,
    TransactionType,
};

/// Builder for XRPL NFTokenAcceptOffer transactions.
///
/// # Example
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::NFTokenAcceptOfferBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = NFTokenAcceptOfferBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
///     .with_nftoken_sell_offer(
///         "A44D2B1668F8B2C634B35AD539E66D9A5A90F2B8A6DBE2A66B0F77892E5A4A3D",
///     )
///     .fill(&client)
///     .await?
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub type NFTokenAcceptOfferBuilder = TransactionBuilder<NFTokenAcceptOffer>;

impl NFTokenAcceptOfferBuilder {
    /// Creates a new `NFTokenAcceptOfferBuilder`; set at least one offer ID before building.
    pub fn new(account: impl AsRef<str>) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            NFTokenAcceptOffer {
                nftoken_sell_offer: None,
                nftoken_buy_offer: None,
                nftoken_broker_fee: None,
            },
        )
    }

    /// Sets the ledger object ID of the sell offer to accept.
    pub fn with_nftoken_sell_offer(mut self, offer: impl AsRef<str>) -> Self {
        self.transaction_type.nftoken_sell_offer =
            Some(offer.as_ref().to_string());
        self
    }

    /// Sets the ledger object ID of the buy offer to accept.
    pub fn with_nftoken_buy_offer(mut self, offer: impl AsRef<str>) -> Self {
        self.transaction_type.nftoken_buy_offer =
            Some(offer.as_ref().to_string());
        self
    }

    /// Sets the broker fee retained when completing a brokered trade.
    pub fn with_nftoken_broker_fee(mut self, fee: impl Into<Amount>) -> Self {
        self.transaction_type.nftoken_broker_fee = Some(fee.into());
        self
    }
}

impl TransactionTypeBuilder for NFTokenAcceptOffer {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(broker_fee) = &self.nftoken_broker_fee {
            validate_amount(broker_fee)?;
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::NFTokenAcceptOffer(self))
    }
}
