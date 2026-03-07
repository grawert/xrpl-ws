mod common;

use serial_test::serial;
use xrpl::subscriptions::ledger::LedgerSubscription;
use xrpl::XrplClient;
use common::*;

#[ignore]
#[serial]
#[tokio::test]
async fn test_ledger_subscription() {
    let client =
        XrplClient::new(SERVER_URL).await.expect("Client creation failed");

    let subscription = LedgerSubscription::new();

    let (_resp, mut receiver) = client
        .subscribe(subscription)
        .await
        .expect("Ledger subscription failed");

    match receiver.receiver().recv().await {
        Ok(ledger) => {
            assert!(ledger.ledger_index > 0);
            assert!(!ledger.ledger_hash.is_empty());
        }
        Err(e) => panic!("Broadcast receiver error: {:?}", e),
    }
}
