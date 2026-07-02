use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::validate_address, transactions::account::DepositPreauth,
    Amount, TransactionType,
};

/// Builder for XRPL DepositPreauth transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::DepositPreauthBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = DepositPreauthBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
///     .with_authorize("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")
///     .fill(&client)
///     .await?
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub type DepositPreauthBuilder = TransactionBuilder<DepositPreauth>;

impl DepositPreauthBuilder {
    /// Creates a new `DepositPreauthBuilder`; set `with_authorize` or `with_unauthorize` before building.
    pub fn new(account: impl AsRef<str>) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            DepositPreauth { authorize: None, unauthorize: None },
        )
    }

    /// Grants deposit authorization to the specified account.
    pub fn with_authorize(mut self, authorize: impl AsRef<str>) -> Self {
        self.transaction_type.authorize = Some(authorize.as_ref().to_string());
        self
    }

    /// Revokes deposit authorization from the specified account.
    pub fn with_unauthorize(mut self, unauthorize: impl AsRef<str>) -> Self {
        self.transaction_type.unauthorize =
            Some(unauthorize.as_ref().to_string());
        self
    }
}

impl TransactionTypeBuilder for DepositPreauth {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(authorize) = &self.authorize {
            validate_address(authorize)?;
        }
        if let Some(unauthorize) = &self.unauthorize {
            validate_address(unauthorize)?;
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::DepositPreauth(self))
    }
}
