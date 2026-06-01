mod common;

use serde_json::json;
use xrpl::Client;
use xrpl::request::ledger::LedgerRequest;
use xrpl::request::ledger_closed::LedgerClosedRequest;
use xrpl::request::ledger_current::LedgerCurrentRequest;
use xrpl::request::ledger_data::LedgerDataRequest;
use common::*;

#[tokio::test]
async fn test_ledger_current() {
    let client = Client::new(server_url());
    let result =
        client.request(LedgerCurrentRequest).await.unwrap().result().unwrap();

    assert!(result.ledger_current_index > 0);
}

#[tokio::test]
async fn test_ledger_closed() {
    let client = Client::new(server_url());
    let result =
        client.request(LedgerClosedRequest).await.unwrap().result().unwrap();

    assert_eq!(result.ledger_hash.len(), 64);
    assert!(result.ledger_index > 0);
}

#[tokio::test]
async fn test_ledger_validated() {
    let client = Client::new(server_url());
    let result = client
        .request(LedgerRequest {
            ledger_index: Some("validated".into()),
            ..Default::default()
        })
        .await
        .unwrap()
        .result()
        .unwrap();

    assert!(result.ledger_hash.as_deref().is_some_and(|h| h.len() == 64));
    assert!(result.ledger_index.is_some_and(|i| i > 0));
    assert!(result.ledger.closed);
    assert!(!result.ledger.total_coins.is_empty());
}

#[tokio::test]
async fn test_ledger_with_transactions() {
    let client = Client::new(server_url());
    let result = client
        .request(LedgerRequest {
            ledger_index: Some("validated".into()),
            transactions: Some(true),
            ..Default::default()
        })
        .await
        .unwrap()
        .result()
        .unwrap();

    // transactions field should be present (may be empty array)
    assert!(result.ledger.transactions.is_some());
}

#[tokio::test]
async fn test_ledger_data_first_page() {
    let client = Client::new(server_url());
    let result = client
        .request(LedgerDataRequest {
            ledger_index: Some(json!("validated")),
            limit: Some(5),
            ..Default::default()
        })
        .await
        .unwrap()
        .result()
        .unwrap();

    assert_eq!(result.ledger_hash.len(), 64);
    assert!(result.ledger_index > 0);
    assert!(!result.state.is_empty(), "state should contain ledger entries");
    // There are millions of entries so there will always be a next page marker
    assert!(result.marker.is_some());
}
