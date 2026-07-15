use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};

/// Retrieves information about a specific ledger version.
///
/// # Example
/// ```rust
/// use xrpl::request::ledger::LedgerRequest;
///
/// let request = LedgerRequest {
///     ledger_index: Some("validated".into()),
///     transactions: Some(true),
///     ..Default::default()
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
pub struct LedgerRequest {
    /// Ledger hash to target a specific ledger version.
    pub ledger_hash: Option<String>,
    /// Ledger index or shortcut ("validated", "closed", "current").
    pub ledger_index: Option<Value>,
    /// Return full JSON data for transactions and state entries rather than hashes.
    pub full: Option<bool>,
    /// Return information on all accounts in the ledger (very large).
    pub accounts: Option<bool>,
    /// Return information on all transactions.
    pub transactions: Option<bool>,
    /// Return full details of transactions and accounts rather than hashes.
    pub expand: Option<bool>,
    /// Include `owner_funds` field on offer transactions.
    pub owner_funds: Option<bool>,
    /// Return transaction information in binary format.
    pub binary: Option<bool>,
    /// Include queued transactions in the results.
    pub queue: Option<bool>,
    /// (Clio only) Return array of hashes that were added, modified, or deleted.
    pub diff: Option<bool>,
    /// (Admin only) Filter results by ledger entry type.
    #[serde(rename = "type")]
    pub entry_type: Option<String>,
}

impl LedgerRequest {
    /// Creates a new `LedgerRequest` with default values.
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

    /// Sets whether to return full JSON data for transactions and state entries.
    pub fn with_full(mut self, full: bool) -> Self {
        self.full = Some(full);
        self
    }

    /// Sets whether to return information on all accounts in the ledger.
    pub fn with_accounts(mut self, accounts: bool) -> Self {
        self.accounts = Some(accounts);
        self
    }

    /// Sets whether to return information on all transactions.
    pub fn with_transactions(mut self, transactions: bool) -> Self {
        self.transactions = Some(transactions);
        self
    }

    /// Sets whether to return full details instead of hashes.
    pub fn with_expand(mut self, expand: bool) -> Self {
        self.expand = Some(expand);
        self
    }

    /// Sets whether to include `owner_funds` on offer transactions.
    pub fn with_owner_funds(mut self, owner_funds: bool) -> Self {
        self.owner_funds = Some(owner_funds);
        self
    }

    /// Sets whether to return transaction info in binary format.
    pub fn with_binary(mut self, binary: bool) -> Self {
        self.binary = Some(binary);
        self
    }

    /// Sets whether to include queued transactions in the results.
    pub fn with_queue(mut self, queue: bool) -> Self {
        self.queue = Some(queue);
        self
    }

    /// Sets whether to return array of hashes added, modified, or deleted (Clio only).
    pub fn with_diff(mut self, diff: bool) -> Self {
        self.diff = Some(diff);
        self
    }

    /// Filters results by a specific ledger entry type (admin only).
    pub fn with_entry_type(mut self, entry_type: &str) -> Self {
        self.entry_type = Some(entry_type.to_owned());
        self
    }
}

impl XrplRequest for LedgerRequest {
    type Response = XrplResponse<LedgerResponse>;
    const COMMAND: &str = "ledger";
}

/// Response to a `ledger` request.
#[derive(Debug, Deserialize)]
pub struct LedgerResponse {
    /// Ledger header and optional transaction/account data.
    pub ledger: LedgerInfo,
    /// Hash of the ledger version returned.
    pub ledger_hash: Option<String>,
    /// Sequence number of the ledger version returned.
    pub ledger_index: Option<u32>,
    /// Whether the data comes from a validated ledger.
    pub validated: Option<bool>,
    /// Queued transactions affecting this ledger (present when `queue` is `true`).
    pub queue_data: Option<Value>,
}

/// Ledger header fields returned inside a `LedgerResponse`.
#[derive(Debug, Deserialize)]
pub struct LedgerInfo {
    /// Root hash of the account state tree.
    pub account_hash: Option<String>,
    /// A bit-map of flags relating to the closing of this ledger.
    pub close_flags: Option<u32>,
    /// Close time as Ripple epoch seconds.
    pub close_time: Option<u64>,
    /// Close time in human-readable UTC format.
    pub close_time_human: Option<String>,
    /// Rounding applied to the close time, in seconds.
    pub close_time_resolution: Option<u32>,
    /// Close time in ISO 8601 format.
    pub close_time_iso: Option<String>,
    /// Whether the ledger has been closed.
    pub closed: bool,
    /// Unique identifying hash of this ledger version.
    pub ledger_hash: String,
    /// Sequence number of this ledger.
    pub ledger_index: u32,
    /// Close time of the parent ledger as Ripple epoch seconds.
    pub parent_close_time: Option<u64>,
    /// Hash of the immediately preceding ledger.
    pub parent_hash: Option<String>,
    /// Total XRP in existence, in drops.
    pub total_coins: String,
    /// Root hash of the transaction tree.
    pub transaction_hash: String,
    /// Transaction hashes or expanded transaction objects (depending on `expand`).
    pub transactions: Option<Value>,
}
