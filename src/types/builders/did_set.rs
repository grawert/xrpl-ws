use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{transactions::did::DIDSet, Amount, TransactionType};

/// Builder for XRPL DIDSet transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::DIDSetBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = DIDSetBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
///     .with_did_document("646f6375") // hex-encoded DID document
///     .with_uri("68747470733a2f2f6578616d706c652e636f6d2f646964") // hex-encoded URI
///     .fill(&client)
///     .await?
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub type DIDSetBuilder = TransactionBuilder<DIDSet>;

impl DIDSetBuilder {
    /// Creates a new `DIDSetBuilder`; set at least one of the optional fields before building.
    pub fn new(account: impl Into<String>) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            DIDSet { did_document: None, data: None, uri: None },
        )
    }

    /// Sets the hex-encoded W3C DID document.
    pub fn with_did_document(
        mut self,
        did_document: impl Into<String>,
    ) -> Self {
        self.transaction_type.did_document = Some(did_document.into());
        self
    }

    /// Sets the hex-encoded arbitrary data associated with the DID.
    pub fn with_data(mut self, data: impl Into<String>) -> Self {
        self.transaction_type.data = Some(data.into());
        self
    }

    /// Sets the hex-encoded URI pointing to the DID document or related resource.
    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.transaction_type.uri = Some(uri.into());
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
        Ok(TransactionType::DIDSet(self))
    }
}
