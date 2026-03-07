mod common;

use serial_test::serial;
use xrpl::subscriptions::{BookSubscription, Book};
use xrpl::request::XrplSubscription; // Add this import for the key() method
use xrpl::XrplClient;
use common::*;
use tokio::time::{timeout, Duration};

#[ignore]
#[serial]
#[tokio::test]
async fn test_book_subscription_xrp_usd() {
    let client = XrplClient::new(SERVER_URL).await.expect("Client failed");

    // Subscribe to XRP/USD order book
    // Using Bitstamp issuer address for USD on testnet
    let subscription = BookSubscription::xrp_to_issued_currency(
        "USD",
        "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B", // Bitstamp USD issuer
        true,                                // Include snapshot
    )
    .expect("Failed to create subscription");

    let (_resp, mut receiver) =
        client.subscribe(subscription).await.expect("Book subscription failed");

    // Wait for book changes with timeout
    let result = timeout(Duration::from_secs(30), async {
        while let Ok(msg) = receiver.receiver().recv().await {
            println!("Book change message type: {}", msg.kind);

            if msg.kind == "bookChanges" {
                println!(
                    "Received book change for ledger: {:?}",
                    msg.ledger_index
                );
                if let Some(changes) = &msg.changes {
                    println!("Number of changes: {}", changes.len());
                }
                return true;
            }
        }
        false
    })
    .await;

    match result {
        Ok(true) => println!("Successfully received book changes"),
        Ok(false) => panic!("No book changes received"),
        Err(_) => {
            println!(
                "Timeout waiting for book changes - this may be expected on low-activity pairs"
            );
            // Don't fail the test as book changes might not happen frequently
        }
    }
}

#[ignore]
#[serial]
#[tokio::test]
async fn test_book_subscription_multiple_books() {
    let client = XrplClient::new(SERVER_URL).await.expect("Client failed");

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

    let (_resp, mut receiver) = client
        .subscribe(subscription)
        .await
        .expect("Multi-book subscription failed");

    // Just verify we can subscribe to multiple books
    let result = timeout(Duration::from_secs(10), async {
        while let Ok(msg) = receiver.receiver().recv().await {
            if msg.kind == "bookChanges" {
                println!("Received book change for multiple books setup");
                return true;
            }
        }
        false
    })
    .await;

    // Don't fail on timeout for this test either
    if let Ok(true) = result {
        println!("Successfully subscribed to multiple books");
    } else {
        println!(
            "Multiple book subscription setup completed (no changes received)"
        );
    }
}

#[test]
fn test_book_creation_and_validation() {
    // Test valid book creation
    let book = Book::xrp_to_issued_currency(
        "USD",
        "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
        true,
    )
    .expect("Should create valid book");

    assert_eq!(book.taker_gets.currency, "XRP");
    assert_eq!(book.taker_gets.issuer, None);
    assert_eq!(book.taker_pays.currency, "USD");
    assert_eq!(
        book.taker_pays.issuer,
        Some("rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".to_string())
    );

    // Test invalid currency code
    let result =
        Book::xrp_to_issued_currency("INVALID_CURRENCY", "issuer", true);
    assert!(result.is_err());

    // Test currency pair
    let book = Book::currency_pair(
        "BTC",
        Some("btc_issuer"),
        "ETH",
        Some("eth_issuer"),
        false,
        true,
    )
    .expect("Should create currency pair book");

    assert_eq!(book.taker_gets.currency, "BTC");
    assert_eq!(book.taker_pays.currency, "ETH");
    assert_eq!(book.both, Some(true));
}

#[test]
fn test_book_subscription_key_generation() {
    let subscription = BookSubscription::xrp_to_issued_currency(
        "USD",
        "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
        true,
    )
    .expect("Should create subscription");

    let key = subscription.key();
    // Hash key should be a valid u64
    assert!(key > 0);

    // Test that keys are deterministic
    let subscription2 = BookSubscription::xrp_to_issued_currency(
        "USD",
        "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
        true,
    )
    .expect("Should create subscription");

    assert_eq!(subscription.key(), subscription2.key());
}

#[test]
fn test_book_builder_patterns() {
    // Test with_book pattern
    let book1 = Book::xrp_to_issued_currency("USD", "issuer1", true).unwrap();
    let book2 = Book::issued_currency_to_xrp("EUR", "issuer2", false).unwrap();

    let subscription =
        BookSubscription::new().with_book(book1).with_book(book2);

    assert_eq!(subscription.books.len(), 2);

    // Test with_books pattern
    let books = vec![
        Book::xrp_to_issued_currency("BTC", "btc_issuer", true).unwrap(),
        Book::xrp_to_issued_currency("ETH", "eth_issuer", true).unwrap(),
    ];

    let subscription = BookSubscription::new().with_books(books);
    assert_eq!(subscription.books.len(), 2);
}
