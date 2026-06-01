use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Removes the Decentralized Identifier (DID) document associated with the submitting account.
///
/// ```rust
/// use xrpl::types::transactions::did::DIDDelete;
/// let tx = DIDDelete {};
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DIDDelete {
    // This transaction only uses common fields
}

/// Creates or updates the Decentralized Identifier (DID) document for the submitting account.
///
/// All three fields are optional — at least one must be provided. The `did_document`
/// and `data` fields must be hex-encoded.
///
/// ```rust
/// use xrpl::types::transactions::did::DIDSet;
/// let tx = DIDSet {
///     uri: Some("68747470733a2f2f6578616d706c652e636f6d2f646964".to_string()),
///     did_document: None,
///     data: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DIDSet {
    /// Hex-encoded W3C DID document.
    #[serde(rename = "DIDDocument")]
    pub did_document: Option<String>,
    /// Hex-encoded arbitrary data associated with the DID.
    #[serde(rename = "Data")]
    pub data: Option<String>,
    /// Hex-encoded URI pointing to the DID document or related resource.
    #[serde(rename = "URI")]
    pub uri: Option<String>,
}
