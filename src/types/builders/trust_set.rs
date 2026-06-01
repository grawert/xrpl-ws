use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::validate_amount, transactions::trust_set::TrustSet, Amount,
    TransactionType,
};

/// Builder for XRPL trust line (TrustSet) transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # use xrpl::types::Amount;
/// use xrpl::{Client, issued, types::builders::TrustSetBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = TrustSetBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     issued!(1000, "USD", "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"),
/// )
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type TrustSetBuilder = TransactionBuilder<TrustSet>;

impl TrustSetBuilder {
    /// Creates a new `TrustSetBuilder` with the required trust line limit amount.
    pub fn new(
        account: impl Into<String>,
        limit_amount: impl Into<Amount>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            TrustSet {
                limit_amount: limit_amount.into(),
                quality_in: None,
                quality_out: None,
            },
        )
    }

    /// Sets the incoming exchange rate applied to this trust line (in billionths).
    pub fn with_quality_in(mut self, quality_in: u32) -> Self {
        self.transaction_type.quality_in = Some(quality_in);
        self
    }

    /// Sets the outgoing exchange rate applied to this trust line (in billionths).
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
                validate_amount(&self.limit_amount)?;
                Ok(())
            }
            Amount::Xrpl(_) => {
                Err(crate::types::validation::ValidationError::InvalidAmount(
                    "TrustSet limit_amount must be an issued currency"
                        .to_string(),
                )
                .into())
            }
            Amount::Mpt { .. } => {
                Err(crate::types::validation::ValidationError::InvalidAmount(
                    "TrustSet cannot be used with MPTs".to_string(),
                )
                .into())
            }
        }
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::TrustSet(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_set_builder_basic() {
        let trust_set = TrustSetBuilder::new(
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
            Amount::IssuedCurrency {
                value: "1000".to_string(),
                currency: "USD".to_string(),
                issuer: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
            },
        )
        .build()
        .expect("Should build valid trust set");

        if let TransactionType::TrustSet(TrustSet { limit_amount, .. }) =
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
            drops!(1000),
        )
        .build();

        assert!(matches!(result, Err(BuildError::Validation(_))));
    }
}
