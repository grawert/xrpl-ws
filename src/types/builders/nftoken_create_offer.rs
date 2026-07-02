use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::{validate_address, validate_amount},
    transactions::nft::NFTokenCreateOffer,
    Amount, TransactionType,
};

/// Builder for XRPL NFTokenCreateOffer transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # use xrpl::types::Amount;
/// use xrpl::{Client, xrp, types::builders::NFTokenCreateOfferBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = NFTokenCreateOfferBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65",
///     xrp!(1),
/// )
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type NFTokenCreateOfferBuilder = TransactionBuilder<NFTokenCreateOffer>;

impl NFTokenCreateOfferBuilder {
    /// Creates a new `NFTokenCreateOfferBuilder` for the given token and price.
    pub fn new(
        account: impl AsRef<str>,
        nftoken_id: impl AsRef<str>,
        amount: impl Into<Amount>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            NFTokenCreateOffer {
                nftoken_id: nftoken_id.as_ref().to_string(),
                amount: amount.into(),
                owner: None,
                expiration: None,
                destination: None,
            },
        )
    }

    /// Sets the current token owner; required for buy offers where the submitter is not the owner.
    pub fn with_owner(mut self, owner: impl AsRef<str>) -> Self {
        self.transaction_type.owner = Some(owner.as_ref().to_string());
        self
    }

    /// Sets the Ripple-epoch expiration time for the offer.
    pub fn with_expiration(mut self, expiration: u32) -> Self {
        self.transaction_type.expiration = Some(expiration);
        self
    }

    /// Restricts offer acceptance to a specific account.
    pub fn with_destination(mut self, destination: impl AsRef<str>) -> Self {
        self.transaction_type.destination =
            Some(destination.as_ref().to_string());
        self
    }
}

impl TransactionTypeBuilder for NFTokenCreateOffer {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_amount(&self.amount)?;
        if let Some(owner) = &self.owner {
            validate_address(owner)?;
        }
        if let Some(destination) = &self.destination {
            validate_address(destination)?;
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::NFTokenCreateOffer(self))
    }
}
