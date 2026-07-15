use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};
use crate::types::Amount;

/// Retrieves open DEX limit orders (offers) placed by an account.
///
/// Returns each standing offer's bid/ask amounts, quality, and optional expiration.
/// Paginate with `limit` and `marker` for accounts with many open orders.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_offers::AccountOffersRequest;
///
/// let req = AccountOffersRequest { limit: Some(100), ..AccountOffersRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh") };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
pub struct AccountOffersRequest {
    /// Account whose open DEX offers are queried (r-address).
    pub account: String,
    /// 64-hex-character hash of the ledger to query.
    pub ledger_hash: Option<String>,
    /// Ledger to query: a sequence number, or a shortcut such as `"validated"`.
    pub ledger_index: Option<Value>,
    /// Maximum number of offers to return in a single response.
    pub limit: Option<u32>,
    /// Pagination cursor returned by a previous response; pass back to fetch the next page.
    pub marker: Option<Value>,
}

impl AccountOffersRequest {
    /// Creates a new request for the given account.
    ///
    /// # Example
    /// ```rust
    /// use xrpl::request::account_offers::AccountOffersRequest;
    /// let req = AccountOffersRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    /// ```
    pub fn new(account: impl AsRef<str>) -> Self {
        Self { account: account.as_ref().to_string(), ..Default::default() }
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

    /// Sets the maximum number of offers to return.
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the pagination marker.
    pub fn with_marker(mut self, marker: impl Into<Value>) -> Self {
        self.marker = Some(marker.into());
        self
    }
}

impl XrplRequest for AccountOffersRequest {
    type Response = XrplResponse<AccountOffersResponse>;
    const COMMAND: &str = "account_offers";
}

/// Response payload for an [`AccountOffersRequest`].
///
/// Contains the page of open DEX offers for the queried account along with ledger
/// context and a pagination marker for retrieving subsequent pages.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_offers::AccountOffersResponse;
///
/// fn total_offers(resp: &AccountOffersResponse) -> usize {
///     resp.offers.len()
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct AccountOffersResponse {
    /// Account whose offers are returned (r-address).
    pub account: String,
    /// Open DEX limit orders placed by the account.
    pub offers: Vec<AccountOffer>,
    /// Sequence number of the current open ledger (present when querying the open ledger).
    pub ledger_current_index: Option<u32>,
    /// Sequence number of the validated ledger used to answer the request.
    pub ledger_index: Option<u32>,
    /// Hash of the ledger used to answer the request.
    pub ledger_hash: Option<String>,
    /// `true` when the response is based on a validated (immutable) ledger.
    pub validated: Option<bool>,
    /// Pagination cursor; present when more offers remain on the next page.
    pub marker: Option<Value>,
    /// Effective page size applied by the server.
    pub limit: Option<u32>,
}

/// A single open DEX limit order placed by an account.
///
/// Describes what the account is willing to exchange: `taker_gets` is what a taker
/// receives (what the account offers), and `taker_pays` is what the account demands
/// in return (what the taker must provide).
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_offers::AccountOffer;
/// use xrpl::types::Amount;
///
/// fn is_sell_xrp(offer: &AccountOffer) -> bool {
///     // taker_gets XRP means the account is selling XRP
///     matches!(offer.taker_gets, Amount::Xrpl(_))
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct AccountOffer {
    /// Bitfield of offer flags (e.g. `lsfPassive`, `lsfSell`).
    pub flags: u32,
    /// Sequence number of the `OfferCreate` transaction that placed this offer.
    pub seq: u32,
    /// Amount the taker receives when consuming this offer (what the account offers).
    pub taker_gets: Amount,
    /// Amount the taker must pay to consume this offer (what the account demands).
    pub taker_pays: Amount,
    /// Exchange rate (`taker_pays / taker_gets`) as a decimal string; lower is better for takers.
    pub quality: String,
    /// Ripple epoch timestamp after which the offer expires and becomes unfillable.
    pub expiration: Option<u32>,
}
