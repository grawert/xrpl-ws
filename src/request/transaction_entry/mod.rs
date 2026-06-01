use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};
use crate::types::Transaction;

/// Retrieves information on a transaction that is included in a specific ledger.
///
/// Unlike `tx`, this always searches a specific ledger version rather than
/// scanning the ledger history.
#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct TransactionEntryRequest {
    /// Hash of the transaction to look up.
    pub tx_hash: String,
    /// Ledger hash to target a specific ledger version.
    pub ledger_hash: Option<String>,
    /// Ledger index or shortcut ("validated", "closed", "current").
    pub ledger_index: Option<Value>,
}

impl XrplRequest for TransactionEntryRequest {
    type Response = XrplResponse<TransactionEntryResponse>;
    const COMMAND: &str = "transaction_entry";
}

/// Response to a `transaction_entry` request.
#[derive(Debug, Deserialize)]
pub struct TransactionEntryResponse {
    /// The transaction in JSON format.
    pub tx_json: Transaction,
    /// Execution metadata, including `delivered_amount` and affected nodes.
    #[serde(alias = "meta")]
    pub metadata: Value,
    /// Hash of the ledger version that contains the transaction.
    pub ledger_hash: Option<String>,
    /// Sequence number of the ledger version that contains the transaction.
    pub ledger_index: u32,
}
