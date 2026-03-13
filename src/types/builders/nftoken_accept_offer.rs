use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{validation::validate_amount, Amount, TransactionType};

/// Builder for XRPL NFTokenAcceptOffer transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{xrp, drops};
/// use xrpl::types::{Amount, builders::NFTokenAcceptOfferBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let accept_offer = NFTokenAcceptOfferBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         1,
///         drops!(10),
///     )
///     .with_nftoken_sell_offer("offer123".to_string())
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct NFTokenAcceptOffer {
    pub nftoken_sell_offer: Option<String>,
    pub nftoken_buy_offer: Option<String>,
    pub nftoken_broker_fee: Option<Amount>,
}

pub type NFTokenAcceptOfferBuilder = TransactionBuilder<NFTokenAcceptOffer>;

impl NFTokenAcceptOfferBuilder {
    pub fn new(account: String, sequence: u32, fee: Amount) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            NFTokenAcceptOffer {
                nftoken_sell_offer: None,
                nftoken_buy_offer: None,
                nftoken_broker_fee: None,
            },
        )
    }

    pub fn with_nftoken_sell_offer(mut self, offer: String) -> Self {
        self.transaction_type.nftoken_sell_offer = Some(offer);
        self
    }

    pub fn with_nftoken_buy_offer(mut self, offer: String) -> Self {
        self.transaction_type.nftoken_buy_offer = Some(offer);
        self
    }

    pub fn with_nftoken_broker_fee(mut self, fee: Amount) -> Self {
        self.transaction_type.nftoken_broker_fee = Some(fee);
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
        Ok(TransactionType::NFTokenAcceptOffer {
            nftoken_sell_offer: self.nftoken_sell_offer,
            nftoken_buy_offer: self.nftoken_buy_offer,
            nftoken_broker_fee: self.nftoken_broker_fee,
        })
    }
}
