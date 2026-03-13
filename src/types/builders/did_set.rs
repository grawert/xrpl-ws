use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{Amount, TransactionType};

/// Builder for XRPL DIDSet transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{drops};
/// use xrpl::types::{Amount, builders::DIDSetBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let did_set = DIDSetBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         1,
///         drops!(10),
///     )
///     .with_did_document("example_document".to_string())
///     .with_uri("https://example.com/did".to_string())
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct DIDSet {
    pub did_document: Option<String>,
    pub data: Option<String>,
    pub uri: Option<String>,
}

pub type DIDSetBuilder = TransactionBuilder<DIDSet>;

impl DIDSetBuilder {
    pub fn new(account: String, sequence: u32, fee: Amount) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            DIDSet { did_document: None, data: None, uri: None },
        )
    }

    pub fn with_did_document(mut self, did_document: String) -> Self {
        self.transaction_type.did_document = Some(did_document);
        self
    }

    pub fn with_data(mut self, data: String) -> Self {
        self.transaction_type.data = Some(data);
        self
    }

    pub fn with_uri(mut self, uri: String) -> Self {
        self.transaction_type.uri = Some(uri);
        self
    }
}

impl TransactionTypeBuilder for DIDSet {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::DIDSet {
            did_document: self.did_document,
            data: self.data,
            uri: self.uri,
        })
    }
}
