use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::validate_address, transactions::credential::CredentialAccept,
    Amount, TransactionType,
};

/// Builder for XRPL CredentialAccept transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::CredentialAcceptBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let credential_type = hex::encode("license");
/// let tx = CredentialAcceptBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     credential_type,
/// )
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type CredentialAcceptBuilder = TransactionBuilder<CredentialAccept>;

impl CredentialAcceptBuilder {
    /// Creates a new `CredentialAcceptBuilder` for the specified issuer and credential type.
    pub fn new(
        account: impl AsRef<str>,
        issuer: impl AsRef<str>,
        credential_type: impl AsRef<str>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            CredentialAccept {
                credential_type: Some(credential_type.as_ref().to_string()),
                issuer: Some(issuer.as_ref().to_string()),
            },
        )
    }
}

impl TransactionTypeBuilder for CredentialAccept {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(issuer) = &self.issuer {
            validate_address(issuer)?;
        } else {
            return Err(BuildError::Validation(
                crate::types::validation::ValidationError::InvalidAddress(
                    "Missing issuer".to_string(),
                ),
            ));
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::CredentialAccept(self))
    }
}
