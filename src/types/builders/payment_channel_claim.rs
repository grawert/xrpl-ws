use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{validation::validate_amount, Amount, TransactionType};

/// Builder for XRPL PaymentChannelClaim transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{xrp, drops};
/// use xrpl::types::{Amount, builders::PaymentChannelClaimBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let channel_claim = PaymentChannelClaimBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         "5DB01B7FFED6B67E6B0414DED11E051D2EE2B7619CE0EAA6286D67A3A4D5BDB3".to_string(),
///         1,
///         drops!(10),
///     )
///     .with_amount(xrp!(50))
///     .with_balance(xrp!(100))
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct PaymentChannelClaim {
    pub channel: String,
    pub amount: Option<Amount>,
    pub balance: Option<Amount>,
    pub credential_ids: Option<Vec<String>>,
    pub public_key: Option<String>,
    pub signature: Option<String>,
}

pub type PaymentChannelClaimBuilder = TransactionBuilder<PaymentChannelClaim>;

impl PaymentChannelClaimBuilder {
    pub fn new(
        account: String,
        channel: String,
        sequence: u32,
        fee: Amount,
    ) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            PaymentChannelClaim {
                channel,
                amount: None,
                balance: None,
                credential_ids: None,
                public_key: None,
                signature: None,
            },
        )
    }

    pub fn with_amount(mut self, amount: Amount) -> Self {
        self.transaction_type.amount = Some(amount);
        self
    }

    pub fn with_balance(mut self, balance: Amount) -> Self {
        self.transaction_type.balance = Some(balance);
        self
    }

    pub fn with_credential_ids(mut self, credential_ids: Vec<String>) -> Self {
        self.transaction_type.credential_ids = Some(credential_ids);
        self
    }

    pub fn with_public_key(mut self, public_key: String) -> Self {
        self.transaction_type.public_key = Some(public_key);
        self
    }

    pub fn with_signature(mut self, signature: String) -> Self {
        self.transaction_type.signature = Some(signature);
        self
    }
}

impl TransactionTypeBuilder for PaymentChannelClaim {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(amount) = &self.amount {
            validate_amount(amount)?;
        }
        if let Some(balance) = &self.balance {
            validate_amount(balance)?;
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::PaymentChannelClaim {
            channel: self.channel,
            amount: self.amount,
            balance: self.balance,
            credential_ids: self.credential_ids,
            public_key: self.public_key,
            signature: self.signature,
        })
    }
}
