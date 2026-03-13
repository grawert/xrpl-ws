use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{transaction::SignerEntryWrapper, Amount, TransactionType};

/// Builder for XRPL SignerListSet transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{drops};
/// use xrpl::types::{Amount, builders::SignerListSetBuilder, transaction::SignerEntryWrapper};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let signer_list_set = SignerListSetBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         1,
///         drops!(10),
///         2,
///     )
///     .with_signer_entries(vec![])
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct SignerListSet {
    pub signer_quorum: u32,
    pub signer_entries: Option<Vec<SignerEntryWrapper>>,
}

pub type SignerListSetBuilder = TransactionBuilder<SignerListSet>;

impl SignerListSetBuilder {
    pub fn new(
        account: String,
        sequence: u32,
        fee: Amount,
        signer_quorum: u32,
    ) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            SignerListSet { signer_quorum, signer_entries: None },
        )
    }

    pub fn with_signer_entries(
        mut self,
        signer_entries: Vec<SignerEntryWrapper>,
    ) -> Self {
        self.transaction_type.signer_entries = Some(signer_entries);
        self
    }

    pub fn add_signer_entry(
        mut self,
        signer_entry: SignerEntryWrapper,
    ) -> Self {
        self.transaction_type
            .signer_entries
            .get_or_insert_with(Vec::new)
            .push(signer_entry);
        self
    }
}

impl TransactionTypeBuilder for SignerListSet {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::SignerListSet {
            signer_quorum: self.signer_quorum,
            signer_entries: self.signer_entries,
        })
    }
}
