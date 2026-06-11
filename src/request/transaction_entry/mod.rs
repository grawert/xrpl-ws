use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use crate::request::{XrplRequest, XrplResponse};
use crate::types::Transaction;

/// Retrieves information on a transaction that is included in a specific ledger.
///
/// Unlike `tx`, this always searches a specific ledger version rather than
/// scanning the ledger history.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
pub struct TransactionEntryRequest {
    /// Hash of the transaction to look up.
    pub tx_hash: String,
    /// Ledger hash to target a specific ledger version.
    pub ledger_hash: Option<String>,
    /// Ledger index or shortcut ("validated", "closed", "current").
    pub ledger_index: Option<Value>,
}

impl TransactionEntryRequest {
    /// Creates a new request for the given transaction hash.
    ///
    /// # Example
    /// ```rust
    /// use xrpl::request::transaction_entry::TransactionEntryRequest;
    /// let req = TransactionEntryRequest::new("C53ECF838647FA5A4C780377025FEC7999AB4182900F8D65A31BC8CF3439D727");
    /// ```
    pub fn new(tx_hash: impl AsRef<str>) -> Self {
        Self { tx_hash: tx_hash.as_ref().to_string(), ..Default::default() }
    }

    /// Sets the target ledger by its hash.
    pub fn with_ledger_hash(mut self, ledger_hash: impl AsRef<str>) -> Self {
        self.ledger_hash = Some(ledger_hash.as_ref().to_string());
        self
    }

    /// Sets the ledger index or shortcut ("validated", "closed", "current").
    pub fn with_ledger_index(mut self, ledger_index: impl Into<Value>) -> Self {
        self.ledger_index = Some(ledger_index.into());
        self
    }
}

impl XrplRequest for TransactionEntryRequest {
    type Response = XrplResponse<TransactionEntryResponse>;
    const COMMAND: &str = "transaction_entry";
}

/// Response to a `transaction_entry` request.
#[derive(Debug, Deserialize)]
pub struct TransactionEntryResponse {
    /// The transaction in JSON format.
    pub ledger_index: u32,
    pub ledger_hash: String,
    #[serde(rename = "tx_json")]
    pub tx_json: Transaction,
    /// Execution metadata, including `delivered_amount` and affected nodes.
    #[serde(alias = "meta")]
    pub metadata: Value,
}
