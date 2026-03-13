use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{validation::validate_address, Amount, TransactionType};

/// Builder for XRPL DepositPreauth transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{drops};
/// use xrpl::types::{Amount, builders::DepositPreauthBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let deposit_preauth = DepositPreauthBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         1,
///         drops!(10),
///     )
///     .with_authorize("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string())
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct DepositPreauth {
    pub authorize: Option<String>,
    pub unauthorize: Option<String>,
}

pub type DepositPreauthBuilder = TransactionBuilder<DepositPreauth>;

impl DepositPreauthBuilder {
    pub fn new(account: String, sequence: u32, fee: Amount) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            DepositPreauth { authorize: None, unauthorize: None },
        )
    }

    pub fn with_authorize(mut self, authorize: String) -> Self {
        self.transaction_type.authorize = Some(authorize);
        self
    }

    pub fn with_unauthorize(mut self, unauthorize: String) -> Self {
        self.transaction_type.unauthorize = Some(unauthorize);
        self
    }
}

impl TransactionTypeBuilder for DepositPreauth {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(authorize) = &self.authorize {
            validate_address(authorize)?;
        }
        if let Some(unauthorize) = &self.unauthorize {
            validate_address(unauthorize)?;
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::DepositPreauth {
            authorize: self.authorize,
            unauthorize: self.unauthorize,
        })
    }
}
