use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Subject accepts a verifiable credential that was previously issued to them.
///
/// A credential only becomes active after it is accepted; unaccepted credentials
/// remain in a pending state on the ledger.
///
/// ```rust
/// use xrpl::types::transactions::credential::CredentialAccept;
/// let tx = CredentialAccept {
///     credential_type: Some(hex::encode("license")),
///     issuer: Some("rIssuerAccount".to_string()),
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CredentialAccept {
    /// Hex-encoded credential type identifier.
    pub credential_type: Option<String>,
    /// Account that issued the credential.
    pub issuer: Option<String>,
}

/// Issuer creates a verifiable credential for a subject account.
///
/// The credential must be accepted by the subject via `CredentialAccept` before it
/// is considered active. An optional expiration (Ripple epoch) and URI can be attached.
///
/// ```rust
/// use xrpl::types::transactions::credential::CredentialCreate;
/// let tx = CredentialCreate {
///     credential_type: Some(hex::encode("license")),
///     subject: Some("rSubjectAccount".to_string()),
///     expiration: None,
///     uri: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CredentialCreate {
    /// Hex-encoded credential type identifier.
    pub credential_type: Option<String>,
    /// Account the credential is issued to.
    pub subject: Option<String>,
    /// Expiration time in seconds since the Ripple epoch (2000-01-01).
    pub expiration: Option<u32>,
    /// Hex-encoded URI pointing to additional credential metadata.
    #[serde(rename = "URI")]
    pub uri: Option<String>,
}

/// Revokes or deletes a credential from the ledger.
///
/// Can be submitted by either the issuer or the subject. At least one of `subject` or
/// `issuer` must be provided to identify the credential entry.
///
/// ```rust
/// use xrpl::types::transactions::credential::CredentialDelete;
/// let tx = CredentialDelete {
///     credential_type: Some(hex::encode("license")),
///     subject: Some("rSubjectAccount".to_string()),
///     issuer: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CredentialDelete {
    /// Hex-encoded credential type identifier.
    pub credential_type: Option<String>,
    /// Subject account of the credential; required if `issuer` is not provided.
    pub subject: Option<String>,
    /// Issuer account of the credential; required if `subject` is not provided.
    pub issuer: Option<String>,
}
