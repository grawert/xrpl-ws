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
    let result = client
        .request(BookOffersRequest {
            taker_gets: Asset::xrp(),
            taker_pays: Asset::token(
                "USD",
                "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            )
            .unwrap(),
            limit: Some(10),
            ledger_index: Some("validated".into()),
            taker: None,
            ledger_hash: None,
            domain: None,
        })
        .await
        .unwrap()
        .result()
        .unwrap();

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

    let result = client
        .request(BookOffersRequest {
            taker_gets: Asset::token(
                "USD",
                "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            )
            .unwrap(),
            taker_pays: Asset::xrp(),
            limit: Some(10),
            ledger_index: Some("validated".into()),
            taker: None,
            ledger_hash: None,
            domain: None,
        })
        .await
        .unwrap()
        .result()
        .unwrap();

    for offer in &result.offers {
        assert!(!offer.account.is_empty());
    }
}

#[test]
fn test_book_offers_serializes() {
    let req = BookOffersRequest {
        taker_gets: Asset::xrp(),
        taker_pays: Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
            .unwrap(),
        limit: Some(20),
        ledger_index: Some("validated".into()),
        taker: None,
        ledger_hash: None,
        domain: None,
    };
    let json = req.to_value();
    assert_eq!(json["command"], "book_offers");
    assert!(json["taker_gets"].is_object());
    assert!(json["taker_pays"].is_object());
    assert_eq!(json["limit"], 20);
    assert_eq!(json["ledger_index"], "validated");
    assert!(json["taker"].is_null());
}
