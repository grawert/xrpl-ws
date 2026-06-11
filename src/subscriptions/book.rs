use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::request::{XrplRequest, XrplResponse, XrplSubscription};
use crate::types::{validation::validate_currency_code, builders::BuildError};

use super::AccountTransactionMessage;

/// Subscription request for order book updates on the XRPL.
///
/// The `books` stream sends a transaction message whenever a transaction
/// affects a subscribed order book — identical in format to the `transactions`
/// stream messages.
///
/// # Examples
///
/// ```no_run
/// use xrpl::Client;
/// use xrpl::subscriptions::BookSubscription;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let client = Client::new("wss://xrplcluster.com");
///     let sub = BookSubscription::xrp_to_issued_currency(
///         "USD",
///         "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
///         false,
///     )?;
///     let (_resp, mut handle) = client.subscribe(&sub).await?;
///     while let Ok(msg) = handle.recv().await {
///         println!("{}: {}", msg.hash, msg.engine_result);
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug, Serialize, Clone)]
pub struct BookSubscription {
    /// Order books to subscribe to.
    books: Vec<Book>,
}

/// A single order book (currency pair) to subscribe to.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct Book {
    /// Currency that the taker receives (the offer's "gets" side).
    taker_gets: BookCurrency,
    /// Currency that the taker pays (the offer's "pays" side).
    taker_pays: BookCurrency,
    /// When `true`, the subscribe response includes the current order book state.
    snapshot: Option<bool>,
    /// When `true`, subscribe to both directions of the currency pair simultaneously.
    both: Option<bool>,
}

/// Currency and optional issuer for one side of an order book.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct BookCurrency {
    /// Three-character ISO currency code or `"XRP"` for native XRP.
    currency: String,
    /// Issuer account address; omitted for XRP.
    issuer: Option<String>,
}

impl BookSubscription {
    /// Create an empty subscription; add books with [`with_book`](Self::with_book) or [`with_books`](Self::with_books).
    pub fn new() -> Self {
        Self { books: Vec::new() }
    }

    /// Add a single book to the subscription.
    pub fn with_book(mut self, book: impl Into<Book>) -> Self {
        self.books.push(book.into());
        self
    }

    /// Add multiple books to the subscription.
    ///
    /// Accepts any iterable of items convertible into [`Book`].
    pub fn with_books<I, B>(mut self, books: I) -> Self
    where
        I: IntoIterator<Item = B>,
        B: Into<Book>,
    {
        self.books.extend(books.into_iter().map(Into::into));
        self
    }

    /// Subscribe to an XRP-to-issued-currency order book (e.g. XRP/USD).
    pub fn xrp_to_issued_currency(
        currency: &str,
        issuer: &str,
        snapshot: bool,
    ) -> Result<Self, BuildError> {
        let book = Book::xrp_to_issued_currency(currency, issuer, snapshot)?;
        Ok(Self::new().with_book(book))
    }

    /// Subscribe to any currency pair order book.
    pub fn currency_pair(
        gets_currency: &str,
        gets_issuer: Option<&str>,
        pays_currency: &str,
        pays_issuer: Option<&str>,
        snapshot: bool,
        both: bool,
    ) -> Result<Self, BuildError> {
        let book = Book::currency_pair(
            gets_currency,
            gets_issuer,
            pays_currency,
            pays_issuer,
            snapshot,
            both,
        )?;
        Ok(Self::new().with_book(book))
    }
}

impl Default for BookSubscription {
    fn default() -> Self {
        Self::new()
    }
}

impl Book {
    /// Create a book for XRP to an issued currency (e.g. XRP/USD).
    pub fn xrp_to_issued_currency(
        currency: &str,
        issuer: &str,
        snapshot: bool,
    ) -> Result<Self, BuildError> {
        validate_currency_code(currency, true)?;

        Ok(Self {
            taker_gets: BookCurrency {
                currency: "XRP".to_string(),
                issuer: None,
            },
            taker_pays: BookCurrency {
                currency: currency.to_string(),
                issuer: Some(issuer.to_string()),
            },
            snapshot: snapshot.then_some(true),
            both: None,
        })
    }

    /// Create a book for an issued currency to XRP (e.g. USD/XRP).
    pub fn issued_currency_to_xrp(
        currency: &str,
        issuer: &str,
        snapshot: bool,
    ) -> Result<Self, BuildError> {
        validate_currency_code(currency, true)?;

        Ok(Self {
            taker_gets: BookCurrency {
                currency: currency.to_string(),
                issuer: Some(issuer.to_string()),
            },
            taker_pays: BookCurrency {
                currency: "XRP".to_string(),
                issuer: None,
            },
            snapshot: snapshot.then_some(true),
            both: None,
        })
    }

    /// Create a book for any currency pair.
    pub fn currency_pair(
        gets_currency: &str,
        gets_issuer: Option<&str>,
        pays_currency: &str,
        pays_issuer: Option<&str>,
        snapshot: bool,
        both: bool,
    ) -> Result<Self, BuildError> {
        validate_currency_code(gets_currency, true)?;
        validate_currency_code(pays_currency, true)?;

        Ok(Self {
            taker_gets: BookCurrency {
                currency: gets_currency.to_string(),
                issuer: gets_issuer.map(|s| s.to_string()),
            },
            taker_pays: BookCurrency {
                currency: pays_currency.to_string(),
                issuer: pays_issuer.map(|s| s.to_string()),
            },
            snapshot: snapshot.then_some(true),
            both: both.then_some(true),
        })
    }

    /// Subscribe to both sides (buy and sell) of the order book.
    pub fn both_sides(mut self) -> Self {
        self.both = Some(true);
        self
    }

    /// Include a snapshot of the current order book state on subscribe.
    pub fn with_snapshot(mut self) -> Self {
        self.snapshot = Some(true);
        self
    }
}

impl XrplRequest for BookSubscription {
    type Response = XrplResponse<BookSubscriptionResponse>;
    const COMMAND: &str = "subscribe";
}

impl XrplSubscription for BookSubscription {
    type Message = AccountTransactionMessage;
}

/// Initial response returned when subscribing to an order book.
///
/// When `snapshot: true` is set, the response also includes an `offers` array
/// with the current order book state, delivered as part of the subscribe response.
#[derive(Debug, Deserialize)]
pub struct BookSubscriptionResponse {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book_xrp_to_usd() {
        let book = Book::xrp_to_issued_currency(
            "USD",
            "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
            true,
        )
        .unwrap();
        let json = serde_json::to_value(&book).unwrap();

        assert_eq!(json["taker_gets"]["currency"], "XRP");
        assert!(json["taker_gets"].get("issuer").is_none());
        assert_eq!(json["taker_pays"]["currency"], "USD");
        assert_eq!(
            json["taker_pays"]["issuer"],
            "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B"
        );
        assert_eq!(json["snapshot"], true);
        assert!(json.get("both").is_none());
    }

    #[test]
    fn test_book_usd_to_xrp() {
        let book = Book::issued_currency_to_xrp(
            "USD",
            "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
            false,
        )
        .unwrap();
        let json = serde_json::to_value(&book).unwrap();

        assert_eq!(json["taker_gets"]["currency"], "USD");
        assert_eq!(json["taker_pays"]["currency"], "XRP");
        assert!(json["taker_pays"].get("issuer").is_none());
        assert!(json.get("snapshot").is_none());
        assert!(json.get("both").is_none());
    }

    #[test]
    fn test_multiple_books() {
        let book1 =
            Book::xrp_to_issued_currency("USD", "issuer1", true).unwrap();
        let book2 =
            Book::xrp_to_issued_currency("EUR", "issuer2", true).unwrap();
        let sub = BookSubscription::new().with_book(book1).with_book(book2);
        let json = serde_json::to_value(&sub).unwrap();

        assert_eq!(json["books"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_book_currency_pair() {
        let book = Book::currency_pair(
            "BTC",
            Some("btc_issuer"),
            "USD",
            Some("usd_issuer"),
            true,
            false,
        )
        .unwrap();
        let json = serde_json::to_value(&book).unwrap();

        assert_eq!(json["taker_gets"]["currency"], "BTC");
        assert_eq!(json["taker_pays"]["currency"], "USD");
        assert!(json.get("both").is_none());
    }

    #[test]
    fn test_invalid_currency() {
        let result = Book::xrp_to_issued_currency(
            "INVALID_LONG_CURRENCY",
            "issuer",
            true,
        );
        assert!(result.is_err());
    }
}
