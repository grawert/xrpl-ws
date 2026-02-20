use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use super::validate::validate_amount;
use crate::types::{Amount, TransactionType};

pub struct TrustSet {
    pub limit_amount: Amount,
    pub quality_in: Option<u32>,
    pub quality_out: Option<u32>,
}

pub type TrustSetBuilder = TransactionBuilder<TrustSet>;

/// Create a new trust line transaction.
///
/// # Example
/// ```no_run
/// use xrpl::types::Amount;
/// use xrpl::types::builders::TrustSetBuilder;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let trust_set = TrustSetBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///     1,
///     Amount::from(10u64),
///     Amount::issued_currency("1000", "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?,
/// )
/// .build()?;
/// # Ok(())
/// # }
/// ```
impl TrustSetBuilder {
    pub fn new(
        account: String,
        sequence: u32,
        fee: Amount,
        limit_amount: Amount,
    ) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            TrustSet { limit_amount, quality_in: None, quality_out: None },
        )
    }

    pub fn with_quality_in(mut self, quality_in: u32) -> Self {
        self.transaction_type.quality_in = Some(quality_in);
        self
    }

    pub fn with_quality_out(mut self, quality_out: u32) -> Self {
        self.transaction_type.quality_out = Some(quality_out);
        self
    }
}

impl TransactionTypeBuilder for TrustSet {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        match &self.limit_amount {
            Amount::IssuedCurrency { .. } => {
                validate_amount(&self.limit_amount)
            }
            Amount::Xrpl(_) => Err(BuildError::InvalidField(
                "TrustSet limit_amount must be an issued currency".to_string(),
            )),
        }
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        Ok(TransactionType::TrustSet {
            limit_amount: self.limit_amount,
            quality_in: self.quality_in,
            quality_out: self.quality_out,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEQUENCE: u32 = 1;

    #[test]
    fn test_trust_set_builder_basic() {
        let trust_set = TrustSetBuilder::new(
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
            SEQUENCE,
            drops!(10),
            Amount::IssuedCurrency {
                value: "1000".to_string(),
                currency: "USD".to_string(),
                issuer: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
            },
        )
        .build()
        .expect("Should build valid trust set");

        if let TransactionType::TrustSet { limit_amount, .. } =
            trust_set.transaction_type
        {
            if let Amount::IssuedCurrency { currency, value, .. } = limit_amount
            {
                assert_eq!(currency, "USD");
                assert_eq!(value, "1000");
            } else {
                panic!("Expected IssuedCurrency");
            }
        } else {
            panic!("Expected TrustSet transaction type");
        }
    }

    #[test]
    fn test_trust_set_rejects_xrp() {
        let result = TrustSetBuilder::new(
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
            SEQUENCE,
            drops!(10),
            Amount::from(1000u64),
        )
        .build();

        assert!(matches!(result, Err(BuildError::InvalidField(_))));
    }
}
