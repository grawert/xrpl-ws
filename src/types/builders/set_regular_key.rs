use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::validate_address, transactions::account::SetRegularKey, Amount,
    TransactionType,
};

/// Builder for XRPL SetRegularKey transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::SetRegularKeyBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = SetRegularKeyBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
///     .with_regular_key("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")
///     .fill(&client)
///     .await?
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub type SetRegularKeyBuilder = TransactionBuilder<SetRegularKey>;

impl SetRegularKeyBuilder {
    /// Creates a new `SetRegularKeyBuilder`; call `with_regular_key` to set or omit to remove the key.
    pub fn new(account: impl AsRef<str>) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            SetRegularKey { regular_key: None },
        )
    }

    /// Sets the alternate signing key to assign to the account.
    pub fn with_regular_key(mut self, regular_key: impl AsRef<str>) -> Self {
        self.transaction_type.regular_key =
            Some(regular_key.as_ref().to_string());
        self
    }
}

impl TransactionTypeBuilder for SetRegularKey {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(regular_key) = &self.regular_key {
            validate_address(regular_key)?;
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::SetRegularKey(self))
    }
}
