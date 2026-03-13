use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{Amount, TransactionType};

/// Builder for XRPL NFTokenCancelOffer transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{drops};
/// use xrpl::types::{Amount, builders::NFTokenCancelOfferBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let cancel_offer = NFTokenCancelOfferBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         1,
///         drops!(10),
///         vec!["offer1".to_string(), "offer2".to_string()],
///     )
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct NFTokenCancelOffer {
    pub nftoken_offers: Vec<String>,
}

pub type NFTokenCancelOfferBuilder = TransactionBuilder<NFTokenCancelOffer>;

impl NFTokenCancelOfferBuilder {
    pub fn new(
        account: String,
        sequence: u32,
        fee: Amount,
        nftoken_offers: Vec<String>,
    ) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            NFTokenCancelOffer { nftoken_offers },
        )
    }

    pub fn add_offer(mut self, offer: String) -> Self {
        self.transaction_type.nftoken_offers.push(offer);
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
        Ok(TransactionType::NFTokenCancelOffer {
            nftoken_offers: self.nftoken_offers,
        })
    }
}
