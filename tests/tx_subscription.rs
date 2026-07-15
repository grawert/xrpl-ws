mod common;

use serial_test::serial;
use tokio::time::{timeout, Duration};
use xrpl::subscriptions::TransactionsSubscription;
use xrpl::types::builders::{PaymentBuilder, SubmitRequestBuilder};
use xrpl::{Client, SubscriptionEvent, xrp};
use common::*;

#[serial]
#[tokio::test]
async fn test_global_transaction_subscription() {
    let client = Client::new(server_url());
    let sub_req = TransactionsSubscription::proposed();
    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, mut stream) =
        conn.subscribe(&sub_req).await.expect("Subscription failed");
    let mut tx_count = 0;
    let mut validated_count: u32 = 0;
    const TARGET_TX_COUNT: usize = 10;

    while tx_count < TARGET_TX_COUNT {
        tx_count += 1;

        match stream.recv().await {
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
                eprintln!("Error receiving message: {err:?}");
                break;
            }
        }
    }

    eprintln!(
        "Successfully processed {tx_count} transactions with {validated_count} validated"
    );
}

/// Submits a payment from the shared test wallet and returns its transaction
/// hash, read directly from the submit response - no need to wait for a
/// push to learn it.
async fn submit_test_payment(client: &Client) -> String {
    let wallet = sender_wallet();
    let sender_address = wallet.public_key.derive_address();
    let destination_address = receiver_address();

    let payment =
        PaymentBuilder::new(sender_address, destination_address, xrp!(1.01))
            .fill(client)
            .await
            .expect("Failed to auto-fill payment")
            .build()
            .expect("Failed to build payment");

    let submit = SubmitRequestBuilder::new(&payment, &wallet)
        .build()
        .expect("Failed to build submit request");

    let response = client
        .request(&submit)
        .await
        .expect("Failed to submit payment")
        .result()
        .expect("submit should succeed");

    response
        .tx_json
        .as_ref()
        .and_then(|v| v.get("hash"))
        .and_then(|v| v.as_str())
        .expect("submit response must include a hash")
        .to_string()
}

/// Polls `conn.recv()` until a `transaction` push matching `hash` appears,
/// ignoring any unrelated traffic, or `duration` elapses.
///
/// The global transaction stream on a busy testnet can carry several
/// transactions per second, so matching on a specific hash (rather than
/// asserting total silence) is what lets these tests tell "our own
/// subscription is still/no-longer active" apart from "someone else's
/// unrelated transaction happened to arrive in the same window."
async fn wait_for_hash(
    conn: &mut xrpl::SubscriptionSession,
    hash: &str,
    duration: Duration,
) -> Result<(), tokio::time::error::Elapsed> {
    timeout(duration, async {
        loop {
            let event =
                conn.recv().await.expect("connection closed unexpectedly");
            if let SubscriptionEvent::Transaction(msg) = event
                && msg.hash == hash
            {
                return;
            }
        }
    })
    .await
}

#[serial]
#[tokio::test]
async fn test_global_transaction_subscription_unsubscribe_stops_pushes() {
    let client = Client::new(server_url());
    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, stream) = conn
        .subscribe(&TransactionsSubscription::proposed())
        .await
        .expect("Subscription failed");

    let hash_a = submit_test_payment(&client).await;
    wait_for_hash(&mut conn, &hash_a, Duration::from_secs(15))
        .await
        .expect("should observe our own transaction while still subscribed");

    stream.unsubscribe().await.expect("unsubscribe should succeed");

    // A transaction submitted *after* unsubscribing must never show up on
    // this connection.
    let hash_b = submit_test_payment(&client).await;
    let result =
        wait_for_hash(&mut conn, &hash_b, Duration::from_secs(10)).await;
    assert!(
        result.is_err(),
        "must not receive a transaction submitted after unsubscribing"
    );
}

#[serial]
#[tokio::test]
async fn test_global_transaction_subscription_drop_stops_pushes() {
    let client = Client::new(server_url());
    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, stream) = conn
        .subscribe(&TransactionsSubscription::proposed())
        .await
        .expect("Subscription failed");

    let hash_a = submit_test_payment(&client).await;
    wait_for_hash(&mut conn, &hash_a, Duration::from_secs(15))
        .await
        .expect("should observe our own transaction while still subscribed");

    drop(stream);
    tokio::time::sleep(Duration::from_millis(500)).await;

    // A transaction submitted *after* dropping the stream must never show
    // up on this connection.
    let hash_b = submit_test_payment(&client).await;
    let result =
        wait_for_hash(&mut conn, &hash_b, Duration::from_secs(10)).await;
    assert!(
        result.is_err(),
        "must not receive a transaction submitted after dropping the stream"
    );
}
