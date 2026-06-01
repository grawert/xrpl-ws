use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::validate_amount, transactions::offer::OfferCreate, Amount,
    TransactionType,
};

/// Builder for XRPL offer (OfferCreate) transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # use xrpl::types::Amount;
/// use xrpl::{Client, xrp, issued, types::builders::OfferCreateBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = OfferCreateBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     xrp!(1.0),
///     issued!(100, "USD", "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"),
/// )
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type OfferCreateBuilder = TransactionBuilder<OfferCreate>;

impl OfferCreateBuilder {
    /// Creates a new `OfferCreateBuilder` with the required taker-gets and taker-pays amounts.
    pub fn new(
        account: impl Into<String>,
        taker_gets: impl Into<Amount>,
        taker_pays: impl Into<Amount>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            OfferCreate {
                taker_gets: taker_gets.into(),
                taker_pays: taker_pays.into(),
                expiration: None,
                offer_sequence: None,
            },
        )
    }

    /// Sets the Ripple-epoch time after which the offer is automatically invalidated.
    pub fn with_expiration(mut self, expiration: u32) -> Self {
        self.transaction_type.expiration = Some(expiration);
        self
    }

    /// Sets the sequence number of an existing offer to cancel when this offer is placed.
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
        self.validate()?;
        Ok(TransactionType::OfferCreate(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offer_create_builder_xrp_to_iou() {
        let offer = OfferCreateBuilder::new(
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
            drops!(1_000_000),
            Amount::IssuedCurrency {
                value: "100".to_string(),
                currency: "USD".to_string(),
                issuer: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
            },
        )
        .build()
        .expect("Should build valid offer");

        if let TransactionType::OfferCreate(OfferCreate {
            taker_gets,
            taker_pays,
            ..
        }) = offer.transaction_type
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
            drops!(1_000_000),
            Amount::IssuedCurrency {
                value: "100".to_string(),
                currency: "USD".to_string(),
                issuer: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
            },
        )
        .with_expiration(12345)
        .build()
        .expect("Should build valid offer with expiration");

        if let TransactionType::OfferCreate(OfferCreate {
            expiration, ..
        }) = offer.transaction_type
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
            drops!(1_000_000),
            Amount::IssuedCurrency {
                value: "100".to_string(),
                currency: "USD".to_string(),
                issuer: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
            },
        )
        .build();

        assert!(matches!(result, Err(BuildError::Validation(_))));
    }
}
