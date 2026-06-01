use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};

/// Retrieves the set of currencies an account can send or receive via trust lines.
///
/// Useful for building currency selectors in wallet UIs or verifying that a trust
/// line exists before issuing a payment in a specific currency.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_currencies::AccountCurrenciesRequest;
///
/// let req = AccountCurrenciesRequest {
///     account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///     ..Default::default()
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct AccountCurrenciesRequest {
    /// Account whose supported currencies are queried (r-address).
    pub account: String,
    /// If `true`, requires the `account` to be a classic address or public key.
    pub strict: Option<bool>,
    /// 64-hex-character hash of the ledger to query.
    pub ledger_hash: Option<String>,
    /// Ledger to query: a sequence number, or a shortcut such as `"validated"`.
    pub ledger_index: Option<Value>,
}

impl XrplRequest for AccountCurrenciesRequest {
    type Response = XrplResponse<AccountCurrenciesResponse>;
    const COMMAND: &str = "account_currencies";
}

/// Response payload for an [`AccountCurrenciesRequest`].
///
/// Lists all currency codes the account can currently send or receive, derived
/// from its active trust lines in the specified ledger.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_currencies::AccountCurrenciesResponse;
///
/// fn can_receive_usd(resp: &AccountCurrenciesResponse) -> bool {
///     resp.receive_currencies.iter().any(|c| c == "USD")
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct AccountCurrenciesResponse {
    /// Hash of the ledger used to answer the request.
    pub ledger_hash: Option<String>,
    /// Sequence number of the ledger used to answer the request.
    pub ledger_index: Option<u32>,
    /// Currency codes the account can receive (3-char ISO or 40-hex non-standard).
    pub receive_currencies: Vec<String>,
    /// Currency codes the account can send (3-char ISO or 40-hex non-standard).
    pub send_currencies: Vec<String>,
    /// `true` when the response is based on a validated (immutable) ledger.
    pub validated: Option<bool>,
}
