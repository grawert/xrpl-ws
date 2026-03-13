mod common;

use serial_test::serial;
use xrpl::subscriptions::TransactionsSubscription;
use xrpl::Client;
use common::*;

#[ignore]
#[serial]
#[tokio::test]
async fn test_global_transaction_subscription() {
    let client = Client::new(&server_url());
    let sub_req = TransactionsSubscription::proposed();
    let (_resp, mut handle) =
        client.subscribe(sub_req).await.expect("Subscription failed");
    let mut tx_count = 0;
    let mut validated_count: u32 = 0;
    const TARGET_TX_COUNT: usize = 10;

    while tx_count < TARGET_TX_COUNT {
        tx_count += 1;

        match handle.recv().await {
            Ok(msg) => {
                eprintln!(
                    "Received tx #{} [{}]: {}",
                    tx_count,
                    if msg.validated { "validated" } else { "unvalidated" },
                    msg.hash
                );

                if msg.validated {
                    assert!(!msg.hash.is_empty());
                    validated_count += 1;
                }
            }
            Err(err) => {
                eprintln!("Error receiving message: {:?}", err);
                break;
            }
        }
    }

    eprintln!(
        "Successfully processed {} transactions with {} validated",
        tx_count, validated_count
    );
}
