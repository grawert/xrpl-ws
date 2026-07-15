use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};
use crate::types::AccountFlags;

/// Retrieves core account state: XRP balance, sequence number, flags, and owner count.
///
/// Optionally includes queued transactions and signer lists. This is the primary
/// request for checking whether an account is funded and for reading its current
/// sequence number before building a transaction.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_info::AccountInfoRequest;
///
/// let req = AccountInfoRequest { queue: Some(true), ..AccountInfoRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh") };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
pub struct AccountInfoRequest {
    /// Account to look up (r-address, base58check encoded).
    pub account: String,
    /// 64-hex-character hash of the ledger to query.
    pub ledger_hash: Option<String>,
    /// Ledger to query: a sequence number, or a shortcut such as `"validated"`.
    pub ledger_index: Option<Value>,
    /// When `true`, include queued transaction data in the response.
    pub queue: Option<bool>,
    /// When `true`, include the account's signer lists in the response.
    pub signer_lists: Option<bool>,
    /// When `true`, only accept a fully-canonical account address (no aliases).
    pub strict: Option<bool>,
}

impl AccountInfoRequest {
    /// Creates a new request for the given account address.
    ///
    /// # Example
    /// ```rust
    /// use xrpl::request::account_info::AccountInfoRequest;
    /// let req = AccountInfoRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    /// ```
    pub fn new(account: impl AsRef<str>) -> Self {
        Self { account: account.as_ref().to_string(), ..Default::default() }
    }

    /// Sets the target ledger hash to query.
    pub fn with_ledger_hash(mut self, hash: impl AsRef<str>) -> Self {
        self.ledger_hash = Some(hash.as_ref().to_string());
        self
    }

    /// Sets the ledger index or shortcut to query.
    pub fn with_ledger_index(mut self, index: impl Into<Value>) -> Self {
        self.ledger_index = Some(index.into());
        self
    }

    /// Configures whether to include queued transaction data.
    pub fn with_queue(mut self, queue: bool) -> Self {
        self.queue = Some(queue);
        self
    }

    /// Configures whether to include signer lists.
    pub fn with_signer_lists(mut self, signer_lists: bool) -> Self {
        self.signer_lists = Some(signer_lists);
        self
    }

    /// Configures whether to reject non-canonical account addresses.
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = Some(strict);
        self
    }
}

impl XrplRequest for AccountInfoRequest {
    type Response = XrplResponse<AccountInfoResponse>;
    const COMMAND: &str = "account_info";
}

/// Response payload for an [`AccountInfoRequest`].
///
/// Provides the on-ledger state of the account, including its [`AccountRoot`] object.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_info::AccountInfoResponse;
///
/// fn next_sequence(resp: &AccountInfoResponse) -> u32 {
///     resp.account_data.sequence
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct AccountInfoResponse {
    /// The account's ledger object containing balance, flags, and sequence number.
    pub account_data: AccountRoot,
    /// Signer lists attached to the account; populated when `signer_lists` was `true`.
    pub signer_lists: Option<Vec<String>>,
    /// Sequence number of the current open ledger (present when querying the open ledger).
    pub ledger_current_index: Option<u32>,
    /// Sequence number of the validated ledger used to answer the request.
    pub ledger_index: Option<u32>,
    /// Queued transaction summary; populated when `queue` was `true`.
    pub queue_data: Option<QueueData>,
    /// `true` when the response is based on a validated (immutable) ledger.
    pub validated: Option<bool>,
}

/// The on-ledger `AccountRoot` object for an XRPL account.
///
/// Holds the authoritative state of the account as recorded in a specific ledger:
/// XRP balance, current sequence number, and owner count used to calculate reserves.
/// Fields are PascalCase on the wire (`Account`, `Balance`, `Flags`, ...).
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_info::AccountRoot;
///
/// fn available_xrp_drops(root: &AccountRoot, reserve_drops: u64) -> u64 {
///     root.balance.parse::<u64>().unwrap_or(0).saturating_sub(reserve_drops)
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccountRoot {
    /// XRPL account address (r-address, base58check encoded). Wire: `Account`.
    pub account: String,
    /// XRP balance in drops as a string. Wire: `Balance`.
    pub balance: String,
    /// Active account flags. Wire: `Flags`.
    pub flags: AccountFlags,
    /// Always `"AccountRoot"`. Wire: `LedgerEntryType`.
    pub ledger_entry_type: String,
    /// Number of objects the account owns (affects reserve). Wire: `OwnerCount`.
    pub owner_count: u32,
    /// Transaction ID of the last transaction that modified this account. Wire: `PreviousTxnID`.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: String,
    /// Ledger sequence containing the last modifying transaction. Wire: `PreviousTxnLgrSeq`.
    pub previous_txn_lgr_seq: u32,
    /// Next valid sequence number for transactions from this account. Wire: `Sequence`.
    pub sequence: u32,
    /// Ledger object index (SHA-512Half of account ID). Wire: `index`.
    #[serde(rename = "index")]
    pub index: String,
}

/// Summary of transactions queued for the account but not yet applied to a validated ledger.
///
/// Returned when [`AccountInfoRequest::queue`] is `true`. Useful for determining
/// the next available sequence number when submitting multiple transactions in quick
/// succession without waiting for each to validate.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_info::QueueData;
///
/// fn next_free_sequence(queue: &QueueData, current_seq: u32) -> u32 {
///     queue.highest_sequence.map(|s| s + 1).unwrap_or(current_seq)
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct QueueData {
    /// Number of transactions currently in the queue for this account.
    pub txn_count: u32,
    /// `true` if any queued transaction would change the account's auth settings.
    pub auth_change_queued: Option<bool>,
    /// Lowest sequence number among all queued transactions.
    pub lowest_sequence: Option<u32>,
    /// Highest sequence number among all queued transactions.
    pub highest_sequence: Option<u32>,
    /// Maximum XRP (in drops) that the queued transactions could spend combined.
    pub max_spend_drops_total: Option<String>,
    /// Per-transaction details; present only when the server includes them.
    pub transactions: Option<Vec<QueueTransaction>>,
}

/// Per-transaction detail within [`QueueData`].
///
/// Each entry describes one queued transaction and the resources it would consume
/// if applied to the ledger.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_info::QueueTransaction;
///
/// fn is_high_fee(tx: &QueueTransaction, threshold: u64) -> bool {
///     tx.fee.parse::<u64>().unwrap_or(0) > threshold
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct QueueTransaction {
    /// `true` if this transaction would change the account's signing authority.
    pub auth_change: bool,
    /// Transaction fee in drops as a string.
    pub fee: String,
    /// Fee level relative to the minimum fee (higher means higher priority).
    pub fee_level: String,
    /// Maximum XRP (in drops) this transaction could spend including the fee.
    pub max_spend_drops: String,
    /// Sequence number of this queued transaction.
    pub seq: u32,
}
