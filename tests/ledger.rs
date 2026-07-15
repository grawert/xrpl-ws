mod common;

use xrpl::Client;
use xrpl::request::ledger::LedgerRequest;
use xrpl::request::ledger_closed::LedgerClosedRequest;
use xrpl::request::ledger_current::LedgerCurrentRequest;
use xrpl::request::ledger_data::LedgerDataRequest;
use common::*;

#[tokio::test]
async fn test_ledger_current() {
    let client = Client::new(server_url());
    let result = client
        .request(&LedgerCurrentRequest)
        .await
        .expect("LedgerCurrent request failed")
        .result()
        .expect("Could not get result from LedgerCurrent response");

    assert!(result.ledger_current_index > 0);
}

#[tokio::test]
async fn test_ledger_closed() {
    let client = Client::new(server_url());
    let result = client
        .request(&LedgerClosedRequest)
        .await
        .expect("LedgerClosed request failed")
        .result()
        .expect("Could not get result from LedgerClosed response");

    assert_eq!(result.ledger_hash.len(), 64);
    assert!(result.ledger_index > 0);
}

#[tokio::test]
async fn test_ledger_validated() {
    let client = Client::new(server_url());
    let request = LedgerRequest::new().with_ledger_index("validated");
    let result = client
        .request(&request)
        .await
        .expect("Ledger request (validated) failed")
        .result()
        .expect("Could not get result from Ledger response");

    assert!(result.ledger_hash.as_deref().is_some_and(|h| h.len() == 64));
    assert!(result.ledger_index.is_some_and(|i| i > 0));
    assert!(result.ledger.closed);
    assert!(!result.ledger.total_coins.is_empty());
}

#[tokio::test]
async fn test_ledger_with_transactions() {
    let client = Client::new(server_url());
    let request = LedgerRequest::new()
        .with_ledger_index("validated")
        .with_transactions(true);
    let result = client
        .request(&request)
        .await
        .expect("Ledger request (with transactions) failed")
        .result()
        .expect("Could not get result from Ledger response");

    // transactions field should be present (may be empty array)
    assert!(result.ledger.transactions.is_some());
}

#[tokio::test]
async fn test_ledger_data_first_page() {
    let client = Client::new(server_url());
    let request =
        LedgerDataRequest::new().with_ledger_index("validated").with_limit(5);
    let result = client
        .request(&request)
        .await
        .expect("LedgerData request failed")
        .result()
        .expect("Could not get result from LedgerData response");

    assert_eq!(result.ledger_hash.len(), 64);
    assert!(result.ledger_index > 0);
    assert!(!result.state.is_empty(), "state should contain ledger entries");
    // There are millions of entries so there will always be a next page marker
    assert!(result.marker.is_some());
}
