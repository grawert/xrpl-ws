use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};
use crate::types::{Amount, Asset};

/// Retrieves a list of offers between two assets from the order book.
///
/// # Examples
///
/// Using the constructor for the common case:
/// ```rust
/// use xrpl::request::book_offers::BookOffersRequest;
/// use xrpl::types::Asset;
///
/// let request = BookOffersRequest::new(
///     Asset::xrp(),
///     Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap(),
/// )
/// .with_limit(20)
/// .with_ledger_index("validated");
/// ```
///
/// Using struct literal syntax when all fields must be explicit:
/// ```rust
/// use xrpl::request::book_offers::BookOffersRequest;
/// use xrpl::types::Asset;
///
/// let request = BookOffersRequest {
///     taker_gets: Asset::xrp(),
///     taker_pays: Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap(),
///     limit: Some(20),
///     ledger_index: Some("validated".into()),
///     taker: None,
///     ledger_hash: None,
///     domain: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct BookOffersRequest {
    /// Asset the taker receives (defines one side of the order book).
    pub taker_gets: Asset,
    /// Asset the taker pays (defines the other side of the order book).
    pub taker_pays: Asset,
    /// If provided, return offers from the corresponding permissioned DEX.
    pub domain: Option<String>,
    /// Ledger hash to target a specific ledger version.
    pub ledger_hash: Option<String>,
    /// Ledger index or shortcut ("validated", "closed", "current").
    pub ledger_index: Option<Value>,
    /// Maximum number of offers to return.
    pub limit: Option<u32>,
    /// Account to use as perspective for unfunded offers.
    pub taker: Option<String>,
}

impl BookOffersRequest {
    /// Creates a new request for the order book between two assets.
    ///
    /// All optional fields (`limit`, `ledger_index`, `taker`, `ledger_hash`) default to `None`.
    /// Use the builder methods to set them, or assign the fields directly after construction.
    pub fn new(
        taker_gets: impl Into<Asset>,
        taker_pays: impl Into<Asset>,
    ) -> Self {
        Self {
            taker_gets: taker_gets.into(),
            taker_pays: taker_pays.into(),
            domain: None,
            ledger_hash: None,
            ledger_index: None,
            limit: None,
            taker: None,
        }
    }

    /// Sets the maximum number of offers to return.
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the ledger index or shortcut to query ("validated", "closed", "current", or a number).
    pub fn with_ledger_index(mut self, index: impl Into<Value>) -> Self {
        self.ledger_index = Some(index.into());
        self
    }

    /// Sets the account whose perspective is used for computing unfunded offer amounts.
    pub fn with_taker(mut self, taker: impl AsRef<str>) -> Self {
        self.taker = Some(taker.as_ref().to_string());
        self
    }

    /// Targets a specific ledger version by its 64-character hex hash.
    pub fn with_ledger_hash(mut self, hash: impl AsRef<str>) -> Self {
        self.ledger_hash = Some(hash.as_ref().to_string());
        self
    }

    /// Sets the optional `domain` field for permissioned DEXs.
    pub fn with_domain(mut self, domain: impl AsRef<str>) -> Self {
        self.domain = Some(domain.as_ref().to_string());
        self
    }
}

impl XrplRequest for BookOffersRequest {
    type Response = XrplResponse<BookOffersResponse>;
    const COMMAND: &str = "book_offers";
}

/// Response to a `book_offers` request.
#[derive(Debug, Deserialize)]
pub struct BookOffersResponse {
    /// Ordered list of offers, best quality first.
    pub offers: Vec<BookOffer>,
    /// Sequence number of the current open ledger (unvalidated results).
    pub ledger_current_index: Option<u32>,
    /// Sequence number of the ledger version used.
    pub ledger_index: Option<u32>,
    /// Hash of the ledger version used.
    pub ledger_hash: Option<String>,
    /// Whether the data comes from a validated ledger.
    pub validated: Option<bool>,
}

// Offer objects in book_offers responses use PascalCase (ledger format) for
// core fields, while rippled adds computed extras (quality, owner_funds, etc.)
// in snake_case.
/// A single offer entry from an order book, including rippled-computed quality fields.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BookOffer {
    /// Account that placed the offer.
    pub account: String,
    /// Offer flags bit field.
    pub flags: u32,
    /// Sequence number that identifies the offer on the ledger.
    pub sequence: u32,
    /// Amount the offer creator receives when the offer executes.
    pub taker_gets: Amount,
    /// Amount the offer creator pays when the offer executes.
    pub taker_pays: Amount,
    /// Index of the book directory page containing this offer.
    pub book_directory: Option<String>,
    /// Position of this offer within its book directory page.
    pub book_node: Option<String>,
    /// Ripple epoch timestamp after which the offer expires.
    pub expiration: Option<u32>,
    // Computed extras returned in snake_case
    /// Exchange rate (taker_pays / taker_gets), higher is better for the taker.
    #[serde(rename = "quality")]
    pub quality: Option<String>,
    /// The account's available balance of `taker_gets`. Omitted for XRP.
    #[serde(rename = "owner_funds")]
    pub owner_funds: Option<String>,
    /// Adjusted `taker_gets` after considering `owner_funds`.
    #[serde(rename = "taker_gets_funded")]
    pub taker_gets_funded: Option<Amount>,
    /// Adjusted `taker_pays` after considering `owner_funds`.
    #[serde(rename = "taker_pays_funded")]
    pub taker_pays_funded: Option<Amount>,
}
