use serde::{Deserialize, Serialize};

use super::{XrplRequest, XrplResponse};

/// Returns the sequence number of the current open ledger.
#[derive(Debug, Clone, Default, Serialize)]
pub struct LedgerCurrentRequest;

impl XrplRequest for LedgerCurrentRequest {
    type Response = XrplResponse<LedgerCurrentResponse>;
    const COMMAND: &str = "ledger_current";
}

/// Response to a `ledger_current` request.
#[derive(Debug, Deserialize)]
pub struct LedgerCurrentResponse {
    /// Sequence number of the current open ledger.
    pub ledger_current_index: u32,
}
