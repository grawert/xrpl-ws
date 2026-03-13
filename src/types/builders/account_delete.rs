use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{validation::validate_address, Amount, TransactionType};

/// Builder for XRPL AccountDelete transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{drops};
/// use xrpl::types::{Amount, builders::AccountDeleteBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let account_delete = AccountDeleteBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
///         1,
///         drops!(10),
///     )
///     .with_destination_tag(12345)
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct AccountDelete {
    pub destination: String,
    pub destination_tag: Option<u32>,
    pub credential_ids: Option<Vec<String>>,
}

pub type AccountDeleteBuilder = TransactionBuilder<AccountDelete>;

impl AccountDeleteBuilder {
    pub fn new(
        account: String,
        destination: String,
        sequence: u32,
        fee: Amount,
    ) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            AccountDelete {
                destination,
                destination_tag: None,
                credential_ids: None,
            },
        )
    }

    pub fn with_destination_tag(mut self, tag: u32) -> Self {
        self.transaction_type.destination_tag = Some(tag);
        self
    }

    pub fn with_credential_ids(mut self, credential_ids: Vec<String>) -> Self {
        self.transaction_type.credential_ids = Some(credential_ids);
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
        Ok(TransactionType::AccountDelete {
            destination: self.destination,
            destination_tag: self.destination_tag,
            credential_ids: self.credential_ids,
        })
    }
}
