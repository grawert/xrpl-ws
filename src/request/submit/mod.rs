use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};

/// Submits a signed transaction blob to the XRPL network.
///
/// The `tx_blob` must be a fully signed, hex-encoded transaction.
/// Always verify the result using `validated` status, not just the submission engine code.
///
/// # Example
/// ```rust
/// use xrpl::request::submit::SubmitRequest;
///
/// let request = SubmitRequest::new("1200002200000000...").with_fail_hard(true);
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
pub struct SubmitRequest {
    /// Hex-encoded signed transaction blob.
    pub tx_blob: String,
    /// If true, reject the transaction instead of queuing it when it cannot enter the open ledger.
    pub fail_hard: Option<bool>,
}

impl SubmitRequest {
    /// Creates a new request with the given signed transaction hex blob.
    ///
    /// # Example
    /// ```rust
    /// use xrpl::request::submit::SubmitRequest;
    /// let req = SubmitRequest::new("1200002200000000...");
    /// ```
    pub fn new(tx_blob: impl AsRef<str>) -> Self {
        Self { tx_blob: tx_blob.as_ref().to_string(), fail_hard: None }
    }

    /// Rejects the transaction instead of queuing it when it cannot enter the open ledger.
    pub fn with_fail_hard(mut self, value: bool) -> Self {
        self.fail_hard = Some(value);
        self
    }
}

impl XrplRequest for SubmitRequest {
    type Response = XrplResponse<SubmitResponse>;
    const COMMAND: &str = "submit";
}

/// Response to a `submit` request.
#[derive(Debug, Deserialize)]
pub struct SubmitResponse {
    /// Symbolic result code, e.g. "tesSUCCESS" or "tecNO_DST".
    pub engine_result: String,
    /// Numeric result code corresponding to `engine_result`.
    pub engine_result_code: i64,
    /// Human-readable description of the result.
    pub engine_result_message: String,
    /// Hex-encoded transaction blob as received.
    pub tx_blob: Option<String>,
    /// The complete transaction in JSON format.
    pub tx_json: Option<serde_json::Value>,
    /// Whether the transaction was accepted by the server's processing engine.
    pub accepted: Option<bool>,
    /// Next sequence number that could be used without a gap.
    pub account_sequence_available: Option<u32>,
    /// Next sequence number that will be consumed.
    pub account_sequence_next: Option<u32>,
    /// Whether the transaction was applied to the open ledger.
    pub applied: Option<bool>,
    /// Whether the transaction was broadcast to peers.
    pub broadcast: Option<bool>,
    /// Whether the transaction was kept in the queue or ledger.
    pub kept: Option<bool>,
    /// Whether the transaction was placed in the fee queue.
    pub queued: Option<bool>,
    /// Minimum fee in drops required to enter the current open ledger.
    pub open_ledger_cost: Option<String>,
    /// Sequence number of the most recently validated ledger at submission time.
    pub validated_ledger_index: Option<u32>,
}
