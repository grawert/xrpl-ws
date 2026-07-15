use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::validate_address, transactions::credential::CredentialCreate,
    Amount, TransactionType,
};

/// Builder for XRPL CredentialCreate transactions.
///
/// # Example
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::CredentialCreateBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// // Expiration: 2030-01-01 00:00:00 UTC
/// // XRPL uses the Ripple Epoch (seconds since 2000-01-01),
/// // so subtract 946_684_800 from the Unix timestamp.
/// let expiration: u32 = 1_893_456_000 - 946_684_800;
/// let credential_type = hex::encode("license");
/// let tx = CredentialCreateBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     credential_type,
/// )
/// .with_expiration(expiration)
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type CredentialCreateBuilder = TransactionBuilder<CredentialCreate>;

impl CredentialCreateBuilder {
    /// Creates a new `CredentialCreateBuilder` for the specified subject and credential type.
    pub fn new(
        account: impl AsRef<str>,
        subject: impl AsRef<str>,
        credential_type: impl AsRef<str>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            CredentialCreate {
                credential_type: Some(credential_type.as_ref().to_string()),
                subject: Some(subject.as_ref().to_string()),
                expiration: None,
                uri: None,
            },
        )
    }

    /// Sets the Ripple-epoch expiration time for the credential.
    pub fn with_expiration(mut self, expiration: u32) -> Self {
        self.transaction_type.expiration = Some(expiration);
        self
    }

    /// Sets the hex-encoded URI pointing to additional credential metadata.
    pub fn with_uri(mut self, uri: impl AsRef<str>) -> Self {
        self.transaction_type.uri = Some(uri.as_ref().to_string());
        self
    }
}

impl TransactionTypeBuilder for CredentialCreate {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(subject) = &self.subject {
            validate_address(subject)?;
        } else {
            return Err(BuildError::Validation(
                crate::types::validation::ValidationError::InvalidAddress(
                    "Missing subject".to_string(),
                ),
            ));
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::CredentialCreate(self))
    }
}
