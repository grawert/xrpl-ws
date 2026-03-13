mod common;

use serial_test::serial;
use xrpl::subscriptions::ledger::LedgerSubscription;
use xrpl::Client;
use common::*;

#[ignore]
#[serial]
#[tokio::test]
async fn test_ledger_subscription() {
    let client = Client::new(&server_url());

    let subscription = LedgerSubscription::new();

    let (_resp, mut handle) = client
        .subscribe(subscription)
        .await
        .expect("Ledger subscription failed");

    match handle.recv().await {
        Ok(ledger) => {
            assert!(ledger.ledger_index > 0);
            assert!(!ledger.ledger_hash.is_empty());
        }
        Err(e) => panic!("Broadcast receiver error: {:?}", e),
    }
}
