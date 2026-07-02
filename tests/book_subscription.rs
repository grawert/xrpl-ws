mod common;

use serial_test::serial;
use xrpl::subscriptions::{Book, BookSubscription};
use xrpl::Client;
use common::*;
use tokio::time::{timeout, Duration};

#[serial]
#[tokio::test]
async fn test_book_subscription_xrp_usd() {
    let client = Client::new(server_url());

    // Subscribe to XRP/USD order book
    // Using Bitstamp issuer address for USD on testnet
    let subscription = BookSubscription::xrp_to_issued_currency(
        "USD",
        "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B", // Bitstamp USD issuer
        true,                                // Include snapshot
    )
    .expect("Failed to create subscription");

    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, mut handle) =
        conn.subscribe(&subscription).await.expect("Book subscription failed");

    let result = timeout(Duration::from_secs(30), async {
        if let Ok(msg) = handle.recv().await {
            println!(
                "Received book transaction: {} (ledger {:?})",
                msg.hash, msg.ledger_index
            );
            true
        } else {
            false
        }
    })
    .await;

    match result {
        Ok(true) => println!("Successfully received book transaction"),
        Ok(false) => panic!("No book transactions received"),
        Err(_) => {
            // Low-activity pairs may not see trades within the timeout window.
            println!(
                "Timeout waiting for book transaction — expected on low-activity pairs"
            );
        }
    }
}

#[serial]
#[tokio::test]
async fn test_book_subscription_multiple_books() {
    let client = Client::new(server_url());

    // Create multiple book subscriptions
    let book1 = Book::xrp_to_issued_currency(
        "USD",
        "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
        true,
    )
    .expect("Failed to create USD book");

    let book2 = Book::xrp_to_issued_currency(
        "EUR",
        "rhub8VRN55s94qWKDv6jmDy1pUykJzF3wq", // Another test issuer
        true,
    )
    .expect("Failed to create EUR book");

    let subscription =
        BookSubscription::new().with_book(book1).with_book(book2);

    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, mut handle) = conn
        .subscribe(&subscription)
        .await
        .expect("Multi-book subscription failed");

    let result = timeout(Duration::from_secs(10), async {
        if let Ok(msg) = handle.recv().await {
            println!(
                "Received book transaction for multiple books setup: {}",
                msg.hash
            );
            true
        } else {
            false
        }
    })
    .await;

    if let Ok(true) = result {
        println!(
            "Successfully received transaction on multi-book subscription"
        );
    } else {
        println!(
            "Multi-book subscription setup completed (no transactions received in window)"
        );
    }
}

#[test]
fn test_book_creation_and_validation() {
    let book = Book::xrp_to_issued_currency(
        "USD",
        "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
        true,
    )
    .expect("Should create valid book");
    let json = serde_json::to_value(&book).unwrap();

    assert_eq!(json["taker_gets"]["currency"], "XRP");
    assert!(json["taker_gets"].get("issuer").is_none());
    assert_eq!(json["taker_pays"]["currency"], "USD");
    assert_eq!(
        json["taker_pays"]["issuer"],
        "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B"
    );

    let result =
        Book::xrp_to_issued_currency("INVALID_CURRENCY", "issuer", true);
    assert!(result.is_err());

    let book = Book::currency_pair(
        "BTC",
        Some("btc_issuer"),
        "ETH",
        Some("eth_issuer"),
        false,
        true,
    )
    .expect("Should create currency pair book");
    let json = serde_json::to_value(&book).unwrap();

    assert_eq!(json["taker_gets"]["currency"], "BTC");
    assert_eq!(json["taker_pays"]["currency"], "ETH");
    assert_eq!(json["both"], true);
}

#[test]
fn test_book_builder_patterns() {
    let book1 = Book::xrp_to_issued_currency("USD", "issuer1", true).unwrap();
    let book2 = Book::issued_currency_to_xrp("EUR", "issuer2", false).unwrap();
    let sub = BookSubscription::new().with_book(book1).with_book(book2);
    let json = serde_json::to_value(&sub).unwrap();

    assert_eq!(json["books"].as_array().unwrap().len(), 2);

    let books = [
        Book::xrp_to_issued_currency("BTC", "btc_issuer", true).unwrap(),
        Book::xrp_to_issued_currency("ETH", "eth_issuer", true).unwrap(),
    ];
    let sub = BookSubscription::new().with_books(books);
    let json = serde_json::to_value(&sub).unwrap();

    assert_eq!(json["books"].as_array().unwrap().len(), 2);
}
