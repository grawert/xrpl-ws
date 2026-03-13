use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::{validate_address, validate_amount},
    Amount, TransactionType,
};

/// Builder for XRPL NFTokenCreateOffer transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{xrp, drops};
/// use xrpl::types::{Amount, builders::NFTokenCreateOfferBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let nftoken_create_offer = NFTokenCreateOfferBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65".to_string(),
///         1,
///         drops!(10),
///         xrp!(100),
///     )
///     .with_owner("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string())
///     .with_expiration(1234567890)
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct NFTokenCreateOffer {
    pub nftoken_id: String,
    pub amount: Amount,
    pub owner: Option<String>,
    pub expiration: Option<u32>,
    pub destination: Option<String>,
}

pub type NFTokenCreateOfferBuilder = TransactionBuilder<NFTokenCreateOffer>;

impl NFTokenCreateOfferBuilder {
    pub fn new(
        account: String,
        nftoken_id: String,
        sequence: u32,
        fee: Amount,
        amount: Amount,
    ) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            NFTokenCreateOffer {
                nftoken_id,
                amount,
                owner: None,
                expiration: None,
                destination: None,
            },
        )
    }

    pub fn with_owner(mut self, owner: String) -> Self {
        self.transaction_type.owner = Some(owner);
        self
    }

    pub fn with_expiration(mut self, expiration: u32) -> Self {
        self.transaction_type.expiration = Some(expiration);
        self
    }

    pub fn with_destination(mut self, destination: String) -> Self {
        self.transaction_type.destination = Some(destination);
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
        Ok(TransactionType::NFTokenCreateOffer {
            nftoken_id: self.nftoken_id,
            amount: self.amount,
            owner: self.owner,
            expiration: self.expiration,
            destination: self.destination,
        })
    }
}
