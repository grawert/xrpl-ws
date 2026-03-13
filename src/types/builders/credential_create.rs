use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{validation::validate_address, Amount, TransactionType};

/// Builder for XRPL CredentialCreate transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{drops};
/// use xrpl::types::{Amount, builders::CredentialCreateBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let credential_create = CredentialCreateBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         1,
///         drops!(10),
///     )
///     .with_credential_type("license".to_string())
///     .with_subject("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string())
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct CredentialCreate {
    pub credential_type: Option<String>,
    pub subject: Option<String>,
    pub expiration: Option<u32>,
}

pub type CredentialCreateBuilder = TransactionBuilder<CredentialCreate>;

impl CredentialCreateBuilder {
    pub fn new(account: String, sequence: u32, fee: Amount) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            CredentialCreate {
                credential_type: None,
                subject: None,
                expiration: None,
            },
        )
    }

    pub fn with_credential_type(mut self, credential_type: String) -> Self {
        self.transaction_type.credential_type = Some(credential_type);
        self
    }

    pub fn with_subject(mut self, subject: String) -> Self {
        self.transaction_type.subject = Some(subject);
        self
    }

    pub fn with_expiration(mut self, expiration: u32) -> Self {
        self.transaction_type.expiration = Some(expiration);
        self
    }
}

impl TransactionTypeBuilder for CredentialCreate {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(subject) = &self.subject {
            validate_address(subject)?;
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::CredentialCreate {
            credential_type: self.credential_type,
            subject: self.subject,
            expiration: self.expiration,
        })
    }
}
