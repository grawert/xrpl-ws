use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use super::validate::validate_amount;
use crate::types::{Amount, TransactionType};

pub struct OfferCreate {
    pub taker_gets: Amount,
    pub taker_pays: Amount,
    pub expiration: Option<u32>,
    pub offer_sequence: Option<u32>,
}

pub type OfferCreateBuilder = TransactionBuilder<OfferCreate>;

/// Create a new offer transaction.
///
/// # Example
/// ```no_run
/// use xrpl::types::Amount;
/// use xrpl::types::builders::OfferCreateBuilder;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let offer = OfferCreateBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///     1,
///     Amount::from(10u64),
///     Amount::from(1_000_000u64),
///     Amount::issued_currency("100", "USD", "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")?,
/// )
/// .build()?;
/// # Ok(())
/// # }
/// ```
impl OfferCreateBuilder {
    pub fn new(
        account: String,
        sequence: u32,
        fee: Amount,
        taker_gets: Amount,
        taker_pays: Amount,
    ) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            OfferCreate {
                taker_gets,
                taker_pays,
                expiration: None,
                offer_sequence: None,
            },
        )
    }

    pub fn with_expiration(mut self, expiration: u32) -> Self {
        self.transaction_type.expiration = Some(expiration);
        self
    }

    pub fn with_offer_sequence(mut self, offer_sequence: u32) -> Self {
        self.transaction_type.offer_sequence = Some(offer_sequence);
        self
    }
}

impl TransactionTypeBuilder for OfferCreate {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_amount(&self.taker_gets)?;
        validate_amount(&self.taker_pays)?;
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        Ok(TransactionType::OfferCreate {
            taker_gets: self.taker_gets,
            taker_pays: self.taker_pays,
            expiration: self.expiration,
            offer_sequence: self.offer_sequence,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEQUENCE: u32 = 1;

    #[test]
    fn test_offer_create_builder_xrp_to_iou() {
        let offer = OfferCreateBuilder::new(
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
            SEQUENCE,
            drops!(10),
            Amount::from(1_000_000u64),
            Amount::IssuedCurrency {
                value: "100".to_string(),
                currency: "USD".to_string(),
                issuer: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
            },
        )
        .build()
        .expect("Should build valid offer");

        if let TransactionType::OfferCreate { taker_gets, taker_pays, .. } =
            offer.transaction_type
        {
            assert_eq!(taker_gets, Amount::Xrpl("1000000".to_string()));
            if let Amount::IssuedCurrency { currency, .. } = taker_pays {
                assert_eq!(currency, "USD");
            } else {
                panic!("Expected IssuedCurrency for taker_pays");
            }
        } else {
            panic!("Expected OfferCreate transaction type");
        }
    }

    #[test]
    fn test_offer_create_builder_with_expiration() {
        let offer = OfferCreateBuilder::new(
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
            SEQUENCE,
            drops!(10),
            Amount::from(1_000_000u64),
            Amount::IssuedCurrency {
                value: "100".to_string(),
                currency: "USD".to_string(),
                issuer: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
            },
        )
        .with_expiration(12345)
        .build()
        .expect("Should build valid offer with expiration");

        if let TransactionType::OfferCreate { expiration, .. } =
            offer.transaction_type
        {
            assert_eq!(expiration, Some(12345));
        } else {
            panic!("Expected OfferCreate transaction type");
        }
    }

    #[test]
    fn test_offer_create_builder_invalid_account() {
        let result = OfferCreateBuilder::new(
            "not_an_address".to_string(),
            SEQUENCE,
            drops!(10),
            Amount::from(1_000_000u64),
            Amount::IssuedCurrency {
                value: "100".to_string(),
                currency: "USD".to_string(),
                issuer: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
            },
        )
        .build();

        assert!(matches!(result, Err(BuildError::InvalidField(_))));
    }
}
