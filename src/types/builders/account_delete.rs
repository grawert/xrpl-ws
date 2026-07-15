use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::validate_address, transactions::account::AccountDelete, Amount,
    TransactionType,
};

/// Builder for XRPL AccountDelete transactions.
///
/// # Example
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::AccountDeleteBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = AccountDeleteBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
/// )
/// .with_destination_tag(12345)
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type AccountDeleteBuilder = TransactionBuilder<AccountDelete>;

impl AccountDeleteBuilder {
    /// Creates a new `AccountDeleteBuilder` with the required destination account.
    pub fn new(account: impl AsRef<str>, destination: impl AsRef<str>) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            AccountDelete {
                destination: destination.as_ref().to_string(),
                destination_tag: None,
                credential_ids: None,
            },
        )
    }

    /// Sets the destination tag for routing within the destination account.
    pub fn with_destination_tag(mut self, tag: u32) -> Self {
        self.transaction_type.destination_tag = Some(tag);
        self
    }

    /// Sets the credential IDs required to pass deposit authorization.
    pub fn with_credential_ids(
        mut self,
        credential_ids: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Self {
        self.transaction_type.credential_ids = Some(
            credential_ids
                .into_iter()
                .map(|s| s.as_ref().to_string())
                .collect(),
        );
        self
    }
}

impl TransactionTypeBuilder for AccountDelete {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_address(&self.destination)?;
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::AccountDelete(self))
    }
}
