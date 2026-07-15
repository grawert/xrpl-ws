use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};

/// Returns all ledger objects in a given ledger version, paginated by marker.
///
/// Used for scanning the entire ledger state. For looking up specific entries
/// prefer `ledger_entry`.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
pub struct LedgerDataRequest {
    /// Ledger hash to target a specific ledger version.
    pub ledger_hash: Option<String>,
    /// Ledger index or shortcut ("validated", "closed", "current").
    pub ledger_index: Option<Value>,
    /// If true, return entries as binary blobs instead of JSON.
    pub binary: Option<bool>,
    /// Maximum number of entries per page.
    pub limit: Option<u32>,
    /// Opaque pagination cursor from a previous response; omit for the first page.
    pub marker: Option<Value>,
    /// Filter results to a specific type of ledger entry.
    #[serde(rename = "type")]
    pub entry_type: Option<String>,
}

impl LedgerDataRequest {
    /// Creates a new `LedgerDataRequest` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the target ledger by its hash.
    pub fn with_ledger_hash(mut self, ledger_hash: &str) -> Self {
        self.ledger_hash = Some(ledger_hash.to_owned());
        self
    }

    /// Sets the ledger index or shortcut ("validated", "closed", "current").
    pub fn with_ledger_index<T: Into<Value>>(
        mut self,
        ledger_index: T,
    ) -> Self {
        self.ledger_index = Some(ledger_index.into());
        self
    }

    /// Sets whether to return entries as binary blobs instead of JSON.
    pub fn with_binary(mut self, binary: bool) -> Self {
        self.binary = Some(binary);
        self
    }

    /// Sets the maximum number of entries per page.
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the pagination marker from a previous response.
    pub fn with_marker<T: Into<Value>>(mut self, marker: T) -> Self {
        self.marker = Some(marker.into());
        self
    }

    /// Filters results to a specific type of ledger entry.
    pub fn with_entry_type(mut self, entry_type: &str) -> Self {
        self.entry_type = Some(entry_type.to_owned());
        self
    }
}

impl XrplRequest for LedgerDataRequest {
    type Response = XrplResponse<LedgerDataResponse>;
    const COMMAND: &str = "ledger_data";
}

/// Response to a `ledger_data` request.
#[derive(Debug, Deserialize)]
pub struct LedgerDataResponse {
    /// The complete ledger header data for this ledger version.
    pub ledger: Option<Value>,
    /// Hash of the ledger version scanned.
    pub ledger_hash: String,
    /// Sequence number of the ledger version scanned.
    pub ledger_index: u32,
    /// Marker for the next page. Absent when the last page has been returned.
    pub marker: Option<Value>,
    /// Ledger entry objects. Each entry includes an `index` field with the entry hash.
    pub state: Vec<Value>,
}
