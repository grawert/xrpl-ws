use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::{validate_address, validate_amount},
    transactions::clawback::Clawback,
    Amount, TransactionType,
};

/// Builder for XRPL Clawback transactions.
///
/// The `amount` must be an issued currency (trust line) or MPT; XRP cannot
/// be clawed back. For trust line tokens, the `issuer` sub-field of `amount`
/// identifies the holder. For MPTs, use `with_holder()` instead.
///
/// # Example
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::{Amount, builders::ClawbackBuilder}};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = ClawbackBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     Amount::IssuedCurrency {
///         value: "100".to_string(),
///         currency: "USD".to_string(),
///         issuer: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
///     },
/// )
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type ClawbackBuilder = TransactionBuilder<Clawback>;

impl ClawbackBuilder {
    /// Creates a new `ClawbackBuilder` with the required reclaim amount.
    pub fn new(account: impl AsRef<str>, amount: impl Into<Amount>) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            Clawback { amount: amount.into(), holder: None },
        )
    }

    /// Sets the MPT holder account to claw back from.
    pub fn with_holder(mut self, holder: impl AsRef<str>) -> Self {
        self.transaction_type.holder = Some(holder.as_ref().to_string());
        self
    }
}

impl TransactionTypeBuilder for Clawback {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_amount(&self.amount)?;
        if let Some(holder) = &self.holder {
            validate_address(holder)?;
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::Clawback(self))
    }
}
