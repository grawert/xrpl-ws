use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{validation::validate_address, Amount, TransactionType};

/// Builder for XRPL CredentialAccept transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{drops};
/// use xrpl::types::{Amount, builders::CredentialAcceptBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let credential_accept = CredentialAcceptBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         1,
///         drops!(10),
///     )
///     .with_credential_type("license".to_string())
///     .with_issuer("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string())
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct CredentialAccept {
    pub credential_type: Option<String>,
    pub issuer: Option<String>,
}

pub type CredentialAcceptBuilder = TransactionBuilder<CredentialAccept>;

impl CredentialAcceptBuilder {
    pub fn new(account: String, sequence: u32, fee: Amount) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            CredentialAccept { credential_type: None, issuer: None },
        )
    }

    pub fn with_credential_type(mut self, credential_type: String) -> Self {
        self.transaction_type.credential_type = Some(credential_type);
        self
    }

    pub fn with_issuer(mut self, issuer: String) -> Self {
        self.transaction_type.issuer = Some(issuer);
        self
    }
}

impl TransactionTypeBuilder for CredentialAccept {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(issuer) = &self.issuer {
            validate_address(issuer)?;
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::CredentialAccept {
            credential_type: self.credential_type,
            issuer: self.issuer,
        })
    }
}
