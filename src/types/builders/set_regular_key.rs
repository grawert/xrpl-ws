use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{validation::validate_address, Amount, TransactionType};

/// Builder for XRPL SetRegularKey transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{drops};
/// use xrpl::types::{Amount, builders::SetRegularKeyBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let set_regular_key = SetRegularKeyBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         1,
///         drops!(10),
///     )
///     .with_regular_key("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string())
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct SetRegularKey {
    pub regular_key: Option<String>,
}

pub type SetRegularKeyBuilder = TransactionBuilder<SetRegularKey>;

impl SetRegularKeyBuilder {
    pub fn new(account: String, sequence: u32, fee: Amount) -> Self {
        Self::init(account, sequence, fee, SetRegularKey { regular_key: None })
    }

    pub fn with_regular_key(mut self, regular_key: String) -> Self {
        self.transaction_type.regular_key = Some(regular_key);
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
        Ok(TransactionType::SetRegularKey { regular_key: self.regular_key })
    }
}
