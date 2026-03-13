use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{validation::validate_address, Amount, TransactionType};

/// Builder for XRPL EscrowFinish transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{drops};
/// use xrpl::types::{Amount, builders::EscrowFinishBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let escrow_finish = EscrowFinishBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
///         1,
///         drops!(10),
///         123,
///     )
///     .with_condition("A0258020E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855810100".to_string())
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct EscrowFinish {
    pub owner: String,
    pub offer_sequence: u32,
    pub condition: Option<String>,
    pub fulfillment: Option<String>,
}

pub type EscrowFinishBuilder = TransactionBuilder<EscrowFinish>;

impl EscrowFinishBuilder {
    pub fn new(
        account: String,
        owner: String,
        sequence: u32,
        fee: Amount,
        offer_sequence: u32,
    ) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            EscrowFinish {
                owner,
                offer_sequence,
                condition: None,
                fulfillment: None,
            },
        )
    }

    pub fn with_condition(mut self, condition: String) -> Self {
        self.transaction_type.condition = Some(condition);
        self
    }

    pub fn with_fulfillment(mut self, fulfillment: String) -> Self {
        self.transaction_type.fulfillment = Some(fulfillment);
        self
    }
}

impl TransactionTypeBuilder for EscrowFinish {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_address(&self.owner)?;
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::EscrowFinish {
            owner: self.owner,
            offer_sequence: self.offer_sequence,
            condition: self.condition,
            fulfillment: self.fulfillment,
        })
    }
}
