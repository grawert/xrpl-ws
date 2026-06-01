use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::validate_address, transactions::account::AccountSet,
    AccountFlag, Amount, TransactionType,
};

/// Builder for XRPL AccountSet transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::AccountSetBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// use xrpl::types::AccountFlag;
/// let tx = AccountSetBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
///     .with_domain("6578616d706c652e636f6d")
///     .with_set_flag(AccountFlag::DefaultRipple)
///     .fill(&client)
///     .await?
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub type AccountSetBuilder = TransactionBuilder<AccountSet>;

impl AccountSetBuilder {
    /// Creates a new `AccountSetBuilder` with no optional fields pre-set.
    pub fn new(account: impl Into<String>) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
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

    /// Clears the given account flag.
    pub fn with_clear_flag(mut self, flag: impl Into<AccountFlag>) -> Self {
        self.transaction_type.clear_flag = Some(flag.into());
        self
    }

    /// Sets the hex-encoded domain name associated with the account.
    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.transaction_type.domain = Some(domain.into());
        self
    }

    /// Sets the MD5 hash of the account's email address for Gravatar lookup.
    pub fn with_email_hash(mut self, hash: impl Into<String>) -> Self {
        self.transaction_type.email_hash = Some(hash.into());
        self
    }

    /// Sets the hex-encoded public key for encrypted messaging.
    pub fn with_message_key(mut self, key: impl Into<String>) -> Self {
        self.transaction_type.message_key = Some(key.into());
        self
    }

    /// Enables the given account flag.
    pub fn with_set_flag(mut self, flag: impl Into<AccountFlag>) -> Self {
        self.transaction_type.set_flag = Some(flag.into());
        self
    }

    /// Sets the transfer rate for issued currencies (in billionths, e.g. 1_005_000_000 = 0.5%).
    pub fn with_transfer_rate(mut self, rate: u32) -> Self {
        self.transaction_type.transfer_rate = Some(rate);
        self
    }

    /// Sets the minimum offer tick size (3–15, or 0 to disable).
    pub fn with_tick_size(mut self, size: u32) -> Self {
        self.transaction_type.tick_size = Some(size);
        self
    }

    /// Sets the account authorized to mint NFTokens on behalf of this account.
    pub fn with_nftoken_minter(mut self, minter: impl Into<String>) -> Self {
        self.transaction_type.nftoken_minter = Some(minter.into());
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
        Ok(TransactionType::AccountSet(self))
    }
}
