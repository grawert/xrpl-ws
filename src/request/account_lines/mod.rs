use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};

/// Retrieves trust lines (IOU balances) for an account.
///
/// Returns the set of trust lines linking the account to other issuers, including
/// balances, limits, and rippling/freeze flags. Paginate with `limit` and `marker`
/// for accounts with many trust lines.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_lines::AccountLinesRequest;
///
/// let req = AccountLinesRequest { limit: Some(200), ..AccountLinesRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh") };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
pub struct AccountLinesRequest {
    /// Account whose trust lines are queried (r-address).
    pub account: String,
    /// When `true`, suppress trust lines that are in their default (zero-balance, default-limit) state.
    pub ignore_default: Option<bool>,
    /// 64-hex-character hash of the ledger to query.
    pub ledger_hash: Option<String>,
    /// Ledger to query: a sequence number, or a shortcut such as `"validated"`.
    pub ledger_index: Option<Value>,
    /// Maximum number of trust lines to return in a single response.
    pub limit: Option<u32>,
    /// Pagination cursor returned by a previous response; pass back to fetch the next page.
    pub marker: Option<Value>,
    /// Restrict results to the trust line with this specific counterparty account (r-address).
    pub peer: Option<String>,
}

impl AccountLinesRequest {
    /// Creates a new request for the given account address.
    ///
    /// # Example
    /// ```rust
    /// use xrpl::request::account_lines::AccountLinesRequest;
    /// let req = AccountLinesRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    /// ```
    pub fn new(account: impl AsRef<str>) -> Self {
        Self { account: account.as_ref().to_string(), ..Default::default() }
    }

    /// Sets whether to ignore default trust lines.
    pub fn with_ignore_default(mut self, value: bool) -> Self {
        self.ignore_default = Some(value);
        self
    }

    /// Sets the ledger hash to query.
    pub fn with_ledger_hash(mut self, hash: impl AsRef<str>) -> Self {
        self.ledger_hash = Some(hash.as_ref().to_string());
        self
    }

    /// Sets the ledger index or shortcut to query.
    pub fn with_ledger_index(mut self, index: impl Into<Value>) -> Self {
        self.ledger_index = Some(index.into());
        self
    }

    /// Sets the maximum number of trust lines to return.
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the pagination marker.
    pub fn with_marker(mut self, marker: impl Into<Value>) -> Self {
        self.marker = Some(marker.into());
        self
    }

    /// Sets the peer account to filter by.
    pub fn with_peer(mut self, peer: impl AsRef<str>) -> Self {
        self.peer = Some(peer.as_ref().to_string());
        self
    }
}

impl XrplRequest for AccountLinesRequest {
    type Response = XrplResponse<AccountLinesResponse>;
    const COMMAND: &str = "account_lines";
}

/// Response payload for an [`AccountLinesRequest`].
///
/// Contains the page of trust lines for the queried account along with ledger
/// context and a pagination marker for retrieving subsequent pages.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_lines::AccountLinesResponse;
///
/// fn usd_balance(resp: &AccountLinesResponse) -> Option<&str> {
///     resp.lines.iter()
///         .find(|l| l.currency == "USD")
///         .map(|l| l.balance.as_str())
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct AccountLinesResponse {
    /// Account whose trust lines are returned (r-address).
    pub account: String,
    /// Trust lines for the account in the queried ledger.
    pub lines: Vec<Trustline>,
    /// Sequence number of the current open ledger (present when querying the open ledger).
    pub ledger_current_index: Option<u32>,
    /// Sequence number of the validated ledger used to answer the request.
    pub ledger_index: Option<u32>,
    /// Hash of the ledger used to answer the request.
    pub ledger_hash: Option<String>,
    /// `true` when the response is based on a validated (immutable) ledger.
    pub validated: Option<bool>,
    /// Pagination cursor; present when more trust lines remain on the next page.
    pub marker: Option<Value>,
    /// Effective page size applied by the server.
    pub limit: Option<u32>,
}

/// A single trust line between two XRPL accounts for an issued currency.
///
/// Describes the bilateral agreement that allows an account to hold an IOU balance
/// from an issuer, including the current balance, trust limits, and rippling/freeze
/// flags on both sides of the line.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_lines::Trustline;
///
/// fn is_frozen(line: &Trustline) -> bool {
///     line.freeze.unwrap_or(false) || line.freeze_peer.unwrap_or(false)
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct Trustline {
    /// Counterparty account address (r-address) on the other side of the trust line.
    pub account: String,
    /// Current IOU balance (positive = account holds, negative = account owes).
    pub balance: String,
    /// Currency code (3-char ISO or 40-hex non-standard).
    pub currency: String,
    /// Maximum IOU balance the account trusts the counterparty to owe.
    pub limit: String,
    /// Maximum IOU balance the counterparty trusts this account to owe.
    pub limit_peer: String,
    /// Inbound quality (exchange rate multiplier) set by this account; 0 means 1:1.
    pub quality_in: u32,
    /// Outbound quality (exchange rate multiplier) set by this account; 0 means 1:1.
    pub quality_out: u32,
    /// `true` if this account has set the NoRipple flag on this trust line.
    pub no_ripple: Option<bool>,
    /// `true` if the counterparty has set the NoRipple flag on this trust line.
    pub no_ripple_peer: Option<bool>,
    /// `true` if this account has authorized the counterparty's trust line.
    pub authorized: Option<bool>,
    /// `true` if the counterparty has authorized this account's trust line.
    pub peer_authorized: Option<bool>,
    /// `true` if this account has frozen the trust line.
    pub freeze: Option<bool>,
    /// `true` if the counterparty has frozen the trust line.
    pub freeze_peer: Option<bool>,
}
