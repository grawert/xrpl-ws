use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use crate::connection::SubscriptionClass;
use crate::request::{XrplRequest, XrplResponse, XrplSubscription};
use crate::types::{
    Transaction, validation::validate_currency_code, builders::BuildError,
};

/// Subscription request for order book updates on the XRPL.
#[derive(Debug, Serialize, Clone)]
pub struct BookSubscription {
    pub books: Vec<Book>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
/// Represents a single order book (currency pair) for subscription.
pub struct Book {
    pub taker_gets: BookCurrency,
    pub taker_pays: BookCurrency,
    pub snapshot: Option<bool>,
    pub both: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
/// Currency and optional issuer for a book side.
pub struct BookCurrency {
    pub currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
}

impl BookSubscription {
    /// Create a new empty BookSubscription.
    pub fn new() -> Self {
        Self { books: Vec::new() }
    }

    /// Add a single book to the subscription.
    pub fn with_book(mut self, book: Book) -> Self {
        self.books.push(book);
        self
    }

    /// Add multiple books to the subscription.
    pub fn with_books<I>(mut self, books: I) -> Self
    where
        I: IntoIterator<Item = Book>,
    {
        self.books.extend(books);
        self
    }

    /// Subscribe to XRP/USD order book
    pub fn xrp_to_issued_currency(
        currency: &str,
        issuer: &str,
        snapshot: bool,
    ) -> Result<Self, BuildError> {
        let book = Book::xrp_to_issued_currency(currency, issuer, snapshot)?;
        Ok(Self::new().with_book(book))
    }

    /// Subscribe to any currency pair
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
    /// Create a book for XRP to issued currency (e.g., XRP/USD)
    pub fn xrp_to_issued_currency(
        currency: &str,
        issuer: &str,
        snapshot: bool,
    ) -> Result<Self, BuildError> {
        validate_currency_code(currency)?;

        Ok(Self {
            taker_gets: BookCurrency {
                currency: "XRP".to_string(),
                issuer: None,
            },
            taker_pays: BookCurrency {
                currency: currency.to_string(),
                issuer: Some(issuer.to_string()),
            },
            snapshot: Some(snapshot),
            both: Some(false),
        })
    }

    /// Create a book for issued currency to XRP (e.g., USD/XRP)
    pub fn issued_currency_to_xrp(
        currency: &str,
        issuer: &str,
        snapshot: bool,
    ) -> Result<Self, BuildError> {
        validate_currency_code(currency)?;

        Ok(Self {
            taker_gets: BookCurrency {
                currency: currency.to_string(),
                issuer: Some(issuer.to_string()),
            },
            taker_pays: BookCurrency {
                currency: "XRP".to_string(),
                issuer: None,
            },
            snapshot: Some(snapshot),
            both: Some(false),
        })
    }

    /// Create a book for any currency pair
    pub fn currency_pair(
        gets_currency: &str,
        gets_issuer: Option<&str>,
        pays_currency: &str,
        pays_issuer: Option<&str>,
        snapshot: bool,
        both: bool,
    ) -> Result<Self, BuildError> {
        validate_currency_code(gets_currency)?;
        validate_currency_code(pays_currency)?;

        Ok(Self {
            taker_gets: BookCurrency {
                currency: gets_currency.to_string(),
                issuer: gets_issuer.map(|s| s.to_string()),
            },
            taker_pays: BookCurrency {
                currency: pays_currency.to_string(),
                issuer: pays_issuer.map(|s| s.to_string()),
            },
            snapshot: Some(snapshot),
            both: Some(both),
        })
    }

    /// Subscribe to both sides (buy and sell) of the book.
    pub fn both_sides(mut self) -> Self {
        self.both = Some(true);
        self
    }

    /// Include a snapshot of the order book in the subscription.
    pub fn with_snapshot(mut self) -> Self {
        self.snapshot = Some(true);
        self
    }
}

impl XrplRequest for BookSubscription {
    type Response = XrplResponse<BookSubscriptionResponse>;
    const COMMAND: &str = "subscribe";
}

impl Hash for BookSubscription {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash each book in a deterministic order
        let mut book_hashes: Vec<u64> = self
            .books
            .iter()
            .map(|book| {
                let mut book_hasher =
                    std::collections::hash_map::DefaultHasher::new();

                // Hash gets currency
                book.taker_gets.currency.hash(&mut book_hasher);
                book.taker_gets.issuer.hash(&mut book_hasher);

                // Hash pays currency
                book.taker_pays.currency.hash(&mut book_hasher);
                book.taker_pays.issuer.hash(&mut book_hasher);

                // Hash options
                book.snapshot.hash(&mut book_hasher);
                book.both.hash(&mut book_hasher);

                book_hasher.finish()
            })
            .collect();

        // Sort hashes for deterministic order
        book_hashes.sort();

        // Hash the sorted book hashes
        "books".hash(state);
        for book_hash in book_hashes {
            book_hash.hash(state);
        }
    }
}

impl XrplSubscription for BookSubscription {
    type Message = BookMessage;

    fn matches(value: &Value) -> bool {
        value.get("type").and_then(|t| t.as_str()) == Some("bookChanges")
    }

    fn subscription_class(&self) -> SubscriptionClass {
        SubscriptionClass::Trading
    }
}

#[derive(Debug, Deserialize)]
pub struct BookSubscriptionResponse {
    // The response for book subscription is typically empty or minimal
    // The real data comes through the subscription messages
}

#[derive(Debug, Clone, Deserialize)]
pub struct BookMessage {
    pub engine_result: Option<String>,
    pub engine_result_code: Option<i32>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<u32>,
    pub ledger_time: Option<u32>,
    pub status: Option<String>,
    pub transaction: Option<Transaction>,
    pub validated: Option<bool>,
    #[serde(rename = "type")]
    pub kind: String,
    pub changes: Option<Vec<BookChange>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BookChange {
    #[serde(rename = "ModifiedNode")]
    pub modified_node: Option<ModifiedNode>,
    #[serde(rename = "CreatedNode")]
    pub created_node: Option<CreatedNode>,
    #[serde(rename = "DeletedNode")]
    pub deleted_node: Option<DeletedNode>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModifiedNode {
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,
    #[serde(rename = "LedgerIndex")]
    pub ledger_index: String,
    #[serde(rename = "FinalFields")]
    pub final_fields: Option<Value>,
    #[serde(rename = "PreviousFields")]
    pub previous_fields: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatedNode {
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,
    #[serde(rename = "LedgerIndex")]
    pub ledger_index: String,
    #[serde(rename = "NewFields")]
    pub new_fields: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeletedNode {
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,
    #[serde(rename = "LedgerIndex")]
    pub ledger_index: String,
    #[serde(rename = "FinalFields")]
    pub final_fields: Option<Value>,
    #[serde(rename = "PreviousFields")]
    pub previous_fields: Option<Value>,
}


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

        assert_eq!(book.taker_gets.currency, "XRP");
        assert_eq!(book.taker_gets.issuer, None);
        assert_eq!(book.taker_pays.currency, "USD");
        assert_eq!(
            book.taker_pays.issuer,
            Some("rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".to_string())
        );
        assert_eq!(book.snapshot, Some(true));
    }

    #[test]
    fn test_book_usd_to_xrp() {
        let book = Book::issued_currency_to_xrp(
            "USD",
            "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
            false,
        )
        .unwrap();

        assert_eq!(book.taker_gets.currency, "USD");
        assert_eq!(book.taker_pays.currency, "XRP");
        assert_eq!(book.taker_pays.issuer, None);
        assert_eq!(book.snapshot, Some(false));
    }

    #[test]
    fn test_book_subscription_key() {
        let subscription = BookSubscription::xrp_to_issued_currency(
            "USD",
            "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
            true,
        )
        .unwrap();

        let key = subscription.key();
        // Hash key should be a valid u64 and deterministic
        assert!(key > 0);

        // Same subscription should produce same hash
        let subscription2 = BookSubscription::xrp_to_issued_currency(
            "USD",
            "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
            true,
        )
        .unwrap();

        assert_eq!(key, subscription2.key());
    }

    #[test]
    fn test_multiple_books() {
        let book1 =
            Book::xrp_to_issued_currency("USD", "issuer1", true).unwrap();
        let book2 =
            Book::xrp_to_issued_currency("EUR", "issuer2", true).unwrap();

        let subscription =
            BookSubscription::new().with_book(book1).with_book(book2);

        assert_eq!(subscription.books.len(), 2);
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

        assert_eq!(book.taker_gets.currency, "BTC");
        assert_eq!(book.taker_pays.currency, "USD");
        assert_eq!(book.both, Some(false));
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

    #[test]
    fn test_book_message_matches() {
        let mut value = serde_json::Map::new();
        value.insert(
            "type".to_string(),
            serde_json::Value::String("bookChanges".to_string()),
        );
        let json_value = serde_json::Value::Object(value);

        assert!(BookSubscription::matches(&json_value));
    }

}
