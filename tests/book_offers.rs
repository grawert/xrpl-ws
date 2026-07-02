mod common;

use xrpl::Client;
use xrpl::request::book_offers::BookOffersRequest;
use xrpl::request::XrplRequest;
use xrpl::types::Asset;
use common::*;

#[tokio::test]
async fn test_book_offers_xrp_to_usd() {
    let client = Client::new(server_url());

    // Query the XRP/USD book. May be empty on testnet but must deserialize cleanly.
    let request = BookOffersRequest::new(
        Asset::xrp(),
        Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
            .expect("Failed to create USD asset"),
    )
    .with_limit(10)
    .with_ledger_index("validated");
    let result = client
        .request(&request)
        .await
        .expect("Failed to request book_offers (XRP/USD)")
        .result()
        .expect("Failed to get book_offers result");

    // The book may be empty, but the response must be valid
    for offer in &result.offers {
        assert!(!offer.account.is_empty());
        if let Some(q) = &offer.quality {
            assert!(!q.is_empty());
        }
    }
}

#[tokio::test]
async fn test_book_offers_usd_to_xrp() {
    let client = Client::new(server_url());

    let request = BookOffersRequest::new(
        Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
            .expect("Failed to create USD asset"),
        Asset::xrp(),
    )
    .with_limit(10)
    .with_ledger_index("validated");
    let result = client
        .request(&request)
        .await
        .expect("Failed to request book_offers (USD/XRP)")
        .result()
        .expect("Failed to get book_offers result");

    for offer in &result.offers {
        assert!(!offer.account.is_empty());
    }
}

#[test]
fn test_book_offers_serializes() {
    let req = BookOffersRequest::new(
        Asset::xrp(),
        Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
            .expect("Failed to create USD asset"),
    )
    .with_limit(20)
    .with_ledger_index("validated");
    let json = req.to_value();
    assert_eq!(json["command"], "book_offers");
    assert!(json["taker_gets"].is_object());
    assert!(json["taker_pays"].is_object());
    assert_eq!(json["limit"], 20);
    assert_eq!(json["ledger_index"], "validated");
    assert!(json["taker"].is_null());
}
