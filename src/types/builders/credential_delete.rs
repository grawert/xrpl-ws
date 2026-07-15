use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::{validate_address, ValidationError},
    transactions::credential::CredentialDelete,
    Amount, TransactionType,
};

/// Builder for XRPL CredentialDelete transactions.
///
/// The caller must provide the `Subject`, `Issuer`, or both to identify the
/// credential entry to delete. `CredentialType` is always required.
///
/// # Example
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::CredentialDeleteBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let credential_type = hex::encode("license");
/// let tx = CredentialDeleteBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     credential_type,
/// )
/// .with_subject("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type CredentialDeleteBuilder = TransactionBuilder<CredentialDelete>;

impl CredentialDeleteBuilder {
    /// Creates a new `CredentialDeleteBuilder` for the given credential type.
    pub fn new(
        account: impl AsRef<str>,
        credential_type: impl AsRef<str>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            CredentialDelete {
                credential_type: Some(credential_type.as_ref().to_string()),
                subject: None,
                issuer: None,
            },
        )
    }

    /// Sets the subject account of the credential to delete.
    pub fn with_subject(mut self, subject: impl AsRef<str>) -> Self {
        self.transaction_type.subject = Some(subject.as_ref().to_string());
        self
    }

    /// Sets the issuer account of the credential to delete.
    pub fn with_issuer(mut self, issuer: impl AsRef<str>) -> Self {
        self.transaction_type.issuer = Some(issuer.as_ref().to_string());
        self
    }
}

impl TransactionTypeBuilder for CredentialDelete {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if self.subject.is_none() && self.issuer.is_none() {
            return Err(ValidationError::InvalidAmount(
                "CredentialDelete requires Subject, Issuer, or both"
                    .to_string(),
            )
            .into());
        }
        if let Some(subject) = &self.subject {
            validate_address(subject)?;
        }
        if let Some(issuer) = &self.issuer {
            validate_address(issuer)?;
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::CredentialDelete(self))
    }
}
