use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};
use crate::types::{HasTransactionMeta, Transaction, TransactionMeta};

/// Looks up a single transaction by its hash or Compact Transaction Identifier (CTID).
///
/// Use `tx_hash` for most lookups. Use `ctid` when you have a compact reference
/// from a validator or receipt. Provide `min_ledger`/`max_ledger` to narrow the
/// search range and reduce server-side cost.
///
/// # Example
/// ```rust
/// use xrpl::request::tx::TxRequest;
///
/// let request = TxRequest::by_hash("E08D6E9754025BA2534A78707605E0601F03ACE063687A0CA1BDDACFCD1698C7")
///     .with_min_ledger(1000)
///     .with_max_ledger(2000);
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
pub struct TxRequest {
    /// Transaction hash (64-character hex).
    #[serde(rename = "transaction")]
    pub tx_hash: Option<String>,
    /// Compact Transaction Identifier (alternative to `tx_hash`).
    pub ctid: Option<String>,
    /// If true, return the transaction in binary format.
    pub binary: Option<bool>,
    /// Earliest ledger sequence to search (inclusive).
    pub min_ledger: Option<u32>,
    /// Latest ledger sequence to search (inclusive).
    pub max_ledger: Option<u32>,
}

impl TxRequest {
    /// Creates a request to look up a transaction by its 64-character hex hash.
    pub fn by_hash(hash: impl AsRef<str>) -> Self {
        Self { tx_hash: Some(hash.as_ref().to_string()), ..Default::default() }
    }

    /// Creates a request to look up a transaction by its Compact Transaction Identifier.
    pub fn by_ctid(ctid: impl AsRef<str>) -> Self {
        Self { ctid: Some(ctid.as_ref().to_string()), ..Default::default() }
    }

    /// Returns the transaction as a binary blob instead of JSON.
    pub fn with_binary(mut self, binary: bool) -> Self {
        self.binary = Some(binary);
        self
    }

    /// Sets the earliest ledger sequence to search (inclusive).
    pub fn with_min_ledger(mut self, min_ledger: u32) -> Self {
        self.min_ledger = Some(min_ledger);
        self
    }

    /// Sets the latest ledger sequence to search (inclusive).
    pub fn with_max_ledger(mut self, max_ledger: u32) -> Self {
        self.max_ledger = Some(max_ledger);
        self
    }
}

impl XrplRequest for TxRequest {
    type Response = XrplResponse<TxResponse>;
    const COMMAND: &str = "tx";
}

/// Response to a `tx` request.
#[derive(Debug, Clone, Deserialize)]
pub struct TxResponse {
    #[serde(flatten)]
    pub transaction: Option<Transaction>,
    /// Compact Transaction Identifier, if available.
    pub ctid: Option<String>,
    /// Transaction hash.
    pub hash: Option<String>,
    /// Hash of the ledger version that contains this transaction.
    pub ledger_hash: Option<String>,
    /// Sequence number of the ledger version that contains this transaction.
    pub ledger_index: Option<u32>,
    /// Execution metadata, including `delivered_amount` and affected nodes.
    pub meta: Option<TransactionMeta>,
    /// Close time of the ledger in which the transaction was applied.
    pub date: Option<u32>,
    /// Whether the transaction is in a validated ledger.
    #[serde(default)]
    pub validated: bool,
    /// The ledger index of the ledger that includes this transaction.
    #[serde(alias = "inLedger")]
    pub in_ledger: Option<u32>,
}

impl HasTransactionMeta for TxResponse {
    fn transaction_meta(&self) -> Option<&TransactionMeta> {
        self.meta.as_ref()
    }
}
