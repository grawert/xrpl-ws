mod common;

use serial_test::serial;
use xrpl::subscriptions::TransactionsSubscription;
use xrpl::XrplClient;
use common::*;

#[ignore]
#[serial]
#[tokio::test]
async fn test_global_transaction_subscription() {
    let client = XrplClient::new(SERVER_URL).await.expect("Client failed");
    let sub_req = TransactionsSubscription::new();
    let (_resp, mut receiver) =
        client.subscribe(sub_req).await.expect("Subscription failed");
    let mut validated_found = false;

    while let Ok(msg) = receiver.receiver().recv().await {
        if msg.validated {
            println!("Caught global validated tx: {}", msg.hash);
            assert!(!msg.hash.is_empty());
            validated_found = true;
            break;
        }
    }

    assert!(
        validated_found,
        "Timed out waiting for any validated transaction on the network"
    );
}
