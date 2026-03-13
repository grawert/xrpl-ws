use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{validation::validate_address, Amount, TransactionType};

/// Builder for XRPL NFTokenBurn transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{drops};
/// use xrpl::types::{Amount, builders::NFTokenBurnBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let burn = NFTokenBurnBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65".to_string(),
///         1,
///         drops!(10),
///     )
///     .with_owner("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string())
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct NFTokenBurn {
    pub nftoken_id: String,
    pub owner: Option<String>,
}

pub type NFTokenBurnBuilder = TransactionBuilder<NFTokenBurn>;

impl NFTokenBurnBuilder {
    pub fn new(
        account: String,
        nftoken_id: String,
        sequence: u32,
        fee: Amount,
    ) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            NFTokenBurn { nftoken_id, owner: None },
        )
    }

    pub fn with_owner(mut self, owner: String) -> Self {
        self.transaction_type.owner = Some(owner);
        self
    }
}

impl TransactionTypeBuilder for NFTokenBurn {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(owner) = &self.owner {
            validate_address(owner)?;
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::NFTokenBurn {
            nftoken_id: self.nftoken_id,
            owner: self.owner,
        })
    }
}
