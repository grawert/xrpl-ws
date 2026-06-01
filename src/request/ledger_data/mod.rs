use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};

/// Returns all ledger objects in a given ledger version, paginated by marker.
///
/// Used for scanning the entire ledger state. For looking up specific entries
/// prefer `ledger_entry`.
#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
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
