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
/// let amount = Amount::issued_currency("100", "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap();
/// let request = RipplePathFindRequest::new("rSource...", "rDest...", amount);
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
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

impl RipplePathFindRequest {
    /// Creates a new request with the mandatory source, destination, and amount fields.
    ///
    /// # Example
    /// ```rust
    /// use xrpl::request::ripple_path_find::RipplePathFindRequest;
    /// use xrpl::types::Amount;
    ///
    /// let amount = Amount::issued_currency("100", "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap();
    /// let req = RipplePathFindRequest::new("rSource...", "rDest...", amount);
    /// ```
    pub fn new(
        source_account: impl AsRef<str>,
        destination_account: impl AsRef<str>,
        destination_amount: impl Into<Amount>,
    ) -> Self {
        Self {
            source_account: source_account.as_ref().to_string(),
            destination_account: destination_account.as_ref().to_string(),
            destination_amount: destination_amount.into(),
            ..Default::default()
        }
    }

    /// If provided, only return paths that use the corresponding permissioned DEX.
    pub fn with_domain(mut self, domain: impl AsRef<str>) -> Self {
        self.domain = Some(domain.as_ref().to_string());
        self
    }

    /// Maximum amount the source account is willing to spend.
    pub fn with_send_max(mut self, send_max: impl Into<Amount>) -> Self {
        self.send_max = Some(send_max.into());
        self
    }

    /// Currencies the source account may use. Defaults to all available.
    pub fn with_source_currencies(
        mut self,
        source_currencies: Vec<Asset>,
    ) -> Self {
        self.source_currencies = Some(source_currencies);
        self
    }

    /// Ledger hash to target a specific ledger version.
    pub fn with_ledger_hash(mut self, hash: impl AsRef<str>) -> Self {
        self.ledger_hash = Some(hash.as_ref().to_string());
        self
    }

    /// Ledger index or shortcut ("validated", "closed", "current").
    pub fn with_ledger_index(mut self, index: impl Into<Value>) -> Self {
        self.ledger_index = Some(index.into());
        self
    }
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
