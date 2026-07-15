use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    transactions::account::SignerListSet, transactions::SignerEntryWrapper,
    validation::validate_address, Amount, TransactionType,
};

/// Builder for XRPL SignerListSet transactions.
///
/// To **delete** an existing signer list, set `signer_quorum` to `0` and do not
/// call `with_signer_entries` or `add_signer_entry` - the field must be omitted
/// entirely (sending an empty array causes `temMALFORMED`).
///
/// # Example
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::{SignerEntry, SignerEntryWrapper, builders::SignerListSetBuilder}};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = SignerListSetBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", 1)
///     .add_signer_entry(SignerEntryWrapper {
///         signer_entry: SignerEntry {
///             account: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
///             signer_weight: 1,
///             wallet_locator: None,
///         },
///     })
///     .fill(&client)
///     .await?
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub type SignerListSetBuilder = TransactionBuilder<SignerListSet>;

impl SignerListSetBuilder {
    /// Creates a new `SignerListSetBuilder` with the required minimum quorum.
    pub fn new(account: impl AsRef<str>, signer_quorum: u32) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            SignerListSet { signer_quorum, signer_entries: None },
        )
    }

    /// Replaces the entire signer list with the given entries.
    pub fn with_signer_entries(
        mut self,
        signer_entries: impl IntoIterator<Item = impl Into<SignerEntryWrapper>>,
    ) -> Self {
        self.transaction_type.signer_entries =
            Some(signer_entries.into_iter().map(Into::into).collect());
        self
    }

    /// Appends a single signer entry to the list.
    pub fn add_signer_entry(
        mut self,
        signer_entry: impl Into<SignerEntryWrapper>,
    ) -> Self {
        self.transaction_type
            .signer_entries
            .get_or_insert_with(Vec::new)
            .push(signer_entry.into());
        self
    }
}

impl TransactionTypeBuilder for SignerListSet {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(entries) = &self.signer_entries {
            for entry in entries {
                validate_address(&entry.signer_entry.account)?;
            }
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::SignerListSet(self))
    }
}
