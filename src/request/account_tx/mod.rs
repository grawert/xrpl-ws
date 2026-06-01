use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};
use crate::types::{HasTransactionMeta, TransactionMeta};

/// Retrieves the transaction history for an account.
///
/// Returns all transactions that affected the account within the specified ledger
/// range. Use `ledger_index_min`/`ledger_index_max` to constrain the search window,
/// `forward` to control chronological order, and `limit`/`marker` to paginate.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_tx::AccountTxRequest;
///
/// let req = AccountTxRequest {
///     account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///     limit: Some(50),
///     forward: Some(true),
///     ..Default::default()
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
pub struct AccountTxRequest {
    /// Account whose transaction history is queried (r-address).
    pub account: String,
    /// Return only transactions of a specific type (e.g. `"AccountSet"`). *Clio server only.*
    pub tx_type: Option<String>,
    /// Earliest ledger sequence to include; `-1` means the oldest available.
    pub ledger_index_min: Option<i64>,
    /// Latest ledger sequence to include; `-1` means the most recent validated ledger.
    pub ledger_index_max: Option<i64>,
    /// 64-hex-character hash identifying a single specific ledger to search.
    pub ledger_hash: Option<String>,
    /// Ledger to query: a sequence number, or a shortcut such as `"validated"`.
    pub ledger_index: Option<Value>,
    /// When `true`, return transactions as raw hex instead of decoded JSON.
    pub binary: Option<bool>,
    /// When `true`, return oldest transactions first (ascending order).
    pub forward: Option<bool>,
    /// Maximum number of transactions to return in a single response.
    pub limit: Option<u32>,
    /// Pagination cursor returned by a previous response; pass back to fetch the next page.
    pub marker: Option<Value>,
}

impl AccountTxRequest {
    /// Creates a new request for the given account address.
    ///
    /// # Example
    /// ```rust
    /// use xrpl::request::account_tx::AccountTxRequest;
    /// let req = AccountTxRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    /// ```
    pub fn new(account: impl Into<String>) -> Self {
        Self { account: account.into(), ..Default::default() }
    }
}

impl XrplRequest for AccountTxRequest {
    type Response = XrplResponse<AccountTxResponse>;
    const COMMAND: &str = "account_tx";
}

/// Response payload for an [`AccountTxRequest`].
///
/// Contains the page of transactions for the queried account along with the actual
/// ledger range searched and a pagination marker for retrieving subsequent pages.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_tx::AccountTxResponse;
///
/// fn has_more_pages(resp: &AccountTxResponse) -> bool {
///     resp.marker.is_some()
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct AccountTxResponse {
    /// Account whose transaction history is returned (r-address).
    pub account: String,
    /// Earliest ledger sequence actually searched (may differ from the requested value).
    pub ledger_index_min: Option<i64>,
    /// Latest ledger sequence actually searched (may differ from the requested value).
    pub ledger_index_max: Option<i64>,
    /// Pagination cursor; present when more transactions remain on the next page.
    pub marker: Option<Value>,
    /// Transactions affecting the account within the searched ledger range.
    pub transactions: Vec<AccountTransaction>,
    /// `true` when the response is based on validated (immutable) ledgers only.
    pub validated: Option<bool>,
    /// Effective page size applied by the server.
    pub limit: Option<u32>,
}

/// A single transaction entry within an [`AccountTxResponse`].
///
/// Pairs the raw transaction JSON with its execution metadata. Always check
/// `validated` before using the data for financial decisions; only validated
/// transactions are final and irreversible.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_tx::AccountTransaction;
/// use xrpl::types::HasTransactionMeta;
/// fn print_delivered(tx: &AccountTransaction) {
///     match tx.delivered_amount() {
///         Some(amount) => println!("Delivered: {amount}"),
///         None => println!("Not a payment transaction"),
///     }
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct AccountTransaction {
    /// The time the ledger containing this transaction was closed, in ISO 8601 format.
    pub close_time_iso: Option<String>,
    /// The unique hash identifier of the transaction.
    pub hash: Option<String>,
    /// A hex string of the ledger version that included this transaction.
    pub ledger_hash: Option<String>,
    /// The ledger index of the ledger version that included this transaction.
    pub ledger_index: Option<u32>,
    /// Transaction execution metadata (JSON mode).
    pub meta: Option<TransactionMeta>,
    /// Transaction execution metadata as a hex string (Binary mode).
    pub meta_blob: Option<String>,
    /// Full transaction object as returned by the server (JSON mode).
    #[serde(alias = "tx", default)]
    pub tx_json: Option<Value>,
    /// A unique hex string defining the transaction (Binary mode).
    pub tx_blob: Option<String>,
    /// `true` when the transaction is in a validated (immutable) ledger.
    #[serde(default)]
    pub validated: bool,
}

impl HasTransactionMeta for AccountTransaction {
    fn transaction_meta(&self) -> Option<&TransactionMeta> {
        self.meta.as_ref()
    }
}

impl AccountTransaction {
    /// Returns the raw transaction flags bitmask, or `0` if not present.
    pub fn flags(&self) -> u32 {
        self.tx_json
            .as_ref()
            .and_then(|tx| tx.get("Flags"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32
    }
}
