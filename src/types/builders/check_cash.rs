use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::{validate_amount, ValidationError},
    transactions::payment::CheckCash,
    Amount, TransactionType,
};

/// Builder for XRPL CheckCash transactions.
///
/// # Example
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # use xrpl::types::Amount;
/// use xrpl::{Client, xrp, types::builders::CheckCashBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = CheckCashBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "838766BA2B995C00744175F69A1B11E32C3DBC40E64801A4056FCBD657F57334",
/// )
/// .with_amount(xrp!(100))
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type CheckCashBuilder = TransactionBuilder<CheckCash>;

impl CheckCashBuilder {
    /// Creates a new `CheckCashBuilder`; set `with_amount` or `with_deliver_min` before building.
    pub fn new(account: impl AsRef<str>, check_id: impl AsRef<str>) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            CheckCash {
                check_id: check_id.as_ref().to_string(),
                amount: None,
                deliver_min: None,
            },
        )
    }

    /// Sets the exact amount to receive; mutually exclusive with `with_deliver_min`.
    pub fn with_amount(mut self, amount: impl Into<Amount>) -> Self {
        self.transaction_type.amount = Some(amount.into());
        self
    }

    /// Sets the minimum amount to receive; mutually exclusive with `with_amount`.
    pub fn with_deliver_min(mut self, deliver_min: impl Into<Amount>) -> Self {
        self.transaction_type.deliver_min = Some(deliver_min.into());
        self
    }
}

impl TransactionTypeBuilder for CheckCash {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        match (&self.amount, &self.deliver_min) {
            (None, None) => {
                return Err(ValidationError::InvalidAmount(
                    "CheckCash requires either Amount or DeliverMin"
                        .to_string(),
                )
                .into());
            }
            (Some(_), Some(_)) => {
                return Err(ValidationError::InvalidAmount(
                    "CheckCash cannot specify both Amount and DeliverMin"
                        .to_string(),
                )
                .into());
            }
            (Some(amount), None) => validate_amount(amount)?,
            (None, Some(deliver_min)) => validate_amount(deliver_min)?,
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::CheckCash(self))
    }
}
