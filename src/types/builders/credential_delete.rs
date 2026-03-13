use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{validation::validate_address, Amount, TransactionType};

/// Builder for XRPL CredentialDelete transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{drops};
/// use xrpl::types::{Amount, builders::CredentialDeleteBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let credential_delete = CredentialDeleteBuilder::new(
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
pub struct CredentialDelete {
    pub credential_type: Option<String>,
    pub subject: Option<String>,
}

pub type CredentialDeleteBuilder = TransactionBuilder<CredentialDelete>;

impl CredentialDeleteBuilder {
    pub fn new(account: String, sequence: u32, fee: Amount) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            CredentialDelete { credential_type: None, subject: None },
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
}

impl TransactionTypeBuilder for CredentialDelete {
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
        Ok(TransactionType::CredentialDelete {
            credential_type: self.credential_type,
            subject: self.subject,
        })
    }
}
