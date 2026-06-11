use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};

/// Submits a multi-signed transaction to the network.
///
/// Use `submit` for single-signed transactions.
///
/// # Example
/// ```rust
/// use xrpl::request::submit_multisigned::SubmitMultisignedRequest;
/// use serde_json::json;
///
/// let request = SubmitMultisignedRequest::new(json!({"TransactionType": "Payment"}))
///     .with_fail_hard(true);
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct SubmitMultisignedRequest {
    /// The fully assembled and multi-signed transaction as a JSON object.
    pub tx_json: Value,
    /// If true, reject the transaction instead of queuing it when it cannot enter the open ledger.
    pub fail_hard: Option<bool>,
}

impl SubmitMultisignedRequest {
    /// Creates a new request with the given transaction JSON.
    pub fn new(tx_json: impl Into<Value>) -> Self {
        Self { tx_json: tx_json.into(), fail_hard: None }
    }

    /// Rejects the transaction instead of queuing it when it cannot enter the open ledger.
    pub fn with_fail_hard(mut self, value: bool) -> Self {
        self.fail_hard = Some(value);
        self
    }
}

impl XrplRequest for SubmitMultisignedRequest {
    type Response = XrplResponse<SubmitMultisignedResponse>;
    const COMMAND: &str = "submit_multisigned";
}

/// Response to a `submit_multisigned` request.
#[skip_serializing_none]
#[derive(Debug, Deserialize)]
pub struct SubmitMultisignedResponse {
    /// Symbolic result code, e.g. "tesSUCCESS" or "tecNO_DST".
    pub engine_result: String,
    /// Numeric result code corresponding to `engine_result`.
    pub engine_result_code: i64,
    /// Human-readable description of the result.
    pub engine_result_message: String,
    /// Hex-encoded transaction blob as received.
    pub tx_blob: Option<String>,
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
    /// The complete transaction in JSON format.
    pub tx_json: Option<Value>,
}
