use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{validation::validate_address, Amount, TransactionType};

/// Builder for XRPL AccountSet transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{drops};
/// use xrpl::types::{Amount, builders::AccountSetBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let account_set = AccountSetBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         1,
///         drops!(10),
///     )
///     .with_domain("example.com".to_string())
///     .with_set_flag(1)
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct AccountSet {
    pub clear_flag: Option<i64>,
    pub domain: Option<String>,
    pub email_hash: Option<String>,
    pub message_key: Option<String>,
    pub set_flag: Option<u32>,
    pub transfer_rate: Option<u32>,
    pub tick_size: Option<u32>,
    pub nftoken_minter: Option<String>,
}

pub type AccountSetBuilder = TransactionBuilder<AccountSet>;

impl AccountSetBuilder {
    pub fn new(account: String, sequence: u32, fee: Amount) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            AccountSet {
                clear_flag: None,
                domain: None,
                email_hash: None,
                message_key: None,
                set_flag: None,
                transfer_rate: None,
                tick_size: None,
                nftoken_minter: None,
            },
        )
    }

    pub fn with_clear_flag(mut self, flag: i64) -> Self {
        self.transaction_type.clear_flag = Some(flag);
        self
    }

    pub fn with_domain(mut self, domain: String) -> Self {
        self.transaction_type.domain = Some(domain);
        self
    }

    pub fn with_email_hash(mut self, hash: String) -> Self {
        self.transaction_type.email_hash = Some(hash);
        self
    }

    pub fn with_message_key(mut self, key: String) -> Self {
        self.transaction_type.message_key = Some(key);
        self
    }

    pub fn with_set_flag(mut self, flag: u32) -> Self {
        self.transaction_type.set_flag = Some(flag);
        self
    }

    pub fn with_transfer_rate(mut self, rate: u32) -> Self {
        self.transaction_type.transfer_rate = Some(rate);
        self
    }

    pub fn with_tick_size(mut self, size: u32) -> Self {
        self.transaction_type.tick_size = Some(size);
        self
    }

    pub fn with_nftoken_minter(mut self, minter: String) -> Self {
        self.transaction_type.nftoken_minter = Some(minter);
        self
    }
}

impl TransactionTypeBuilder for AccountSet {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(nftoken_minter) = &self.nftoken_minter {
            validate_address(nftoken_minter)?;
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::AccountSet {
            clear_flag: self.clear_flag,
            domain: self.domain,
            email_hash: self.email_hash,
            message_key: self.message_key,
            set_flag: self.set_flag,
            transfer_rate: self.transfer_rate,
            tick_size: self.tick_size,
            nftoken_minter: self.nftoken_minter,
        })
    }
}
