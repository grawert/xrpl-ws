use serde::{Deserialize, Serialize};

use super::{XrplRequest, XrplResponse};

/// Returns the unique identifiers of the most recently closed ledger.
#[derive(Debug, Clone, Default, Serialize)]
pub struct LedgerClosedRequest;

impl XrplRequest for LedgerClosedRequest {
    type Response = XrplResponse<LedgerClosedResponse>;
    const COMMAND: &str = "ledger_closed";
}

/// Response to a `ledger_closed` request.
#[derive(Debug, Deserialize)]
pub struct LedgerClosedResponse {
    /// Hash of the most recently closed ledger.
    pub ledger_hash: String,
    /// Sequence number of the most recently closed ledger.
    pub ledger_index: u32,
}
