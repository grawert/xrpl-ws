use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};
use crate::types::{Amount, Asset};

/// Finds a payment path between a source and destination account (single-shot).
///
/// Returns a list of path alternatives sorted by quality. Use the best
/// alternative's `source_amount` when building the `Payment` transaction.
///
/// # Example
/// ```rust
/// use xrpl::request::ripple_path_find::RipplePathFindRequest;
/// use xrpl::types::Amount;
///
/// let request = RipplePathFindRequest {
///     source_account: "rSource...".to_string(),
///     destination_account: "rDest...".to_string(),
///     destination_amount: Amount::issued_currency("100", "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap(),
///     ..Default::default()
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct RipplePathFindRequest {
    /// Account that will send the payment.
    pub source_account: String,
    /// Account that will receive the payment.
    pub destination_account: String,
    /// Amount the destination account should receive.
    pub destination_amount: Amount,
    /// If provided, only return paths that use the corresponding permissioned DEX.
    pub domain: Option<String>,
    /// Maximum amount the source account is willing to spend.
    pub send_max: Option<Amount>,
    /// Currencies the source account may use. Defaults to all available.
    pub source_currencies: Option<Vec<Asset>>,
    /// Ledger hash to target a specific ledger version.
    pub ledger_hash: Option<String>,
    /// Ledger index or shortcut ("validated", "closed", "current").
    pub ledger_index: Option<Value>,
}

impl XrplRequest for RipplePathFindRequest {
    type Response = XrplResponse<RipplePathFindResponse>;
    const COMMAND: &str = "ripple_path_find";
}

/// Response to a `ripple_path_find` request.
#[derive(Debug, Deserialize)]
pub struct RipplePathFindResponse {
    /// Available path alternatives, sorted by quality (best first).
    pub alternatives: Vec<PathAlternative>,
    /// Destination account from the request.
    pub destination_account: String,
    /// Destination amount from the request.
    pub destination_amount: Amount,
    /// Currencies the destination account accepts.
    pub destination_currencies: Option<Vec<String>>,
    /// Source account from the request.
    pub source_account: String,
    /// Whether the response is complete (not a partial streaming update).
    pub full_reply: Option<bool>,
    /// Sequence number of the current open ledger (unvalidated results).
    pub ledger_current_index: Option<u32>,
    /// Hash of the ledger version used.
    pub ledger_hash: Option<String>,
    /// Sequence number of the ledger version used.
    pub ledger_index: Option<u32>,
    /// Whether the data comes from a validated ledger.
    pub validated: Option<bool>,
}

/// A single payment path alternative returned by `ripple_path_find`.
#[derive(Debug, Deserialize)]
pub struct PathAlternative {
    /// Computed payment paths in XRPL path format.
    pub paths_computed: Value,
    /// Amount the source account must send along this path.
    pub source_amount: Amount,
}
