mod common;

use serial_test::serial;
use tokio::time::{timeout, Duration};
use xrpl::types::HasTransactionMeta;
use xrpl::subscriptions::AccountTransactionsSubscription;
use xrpl::types::builders::{PaymentBuilder, SubmitRequestBuilder};
use xrpl::{Client, SubscriptionEvent, xrp};
use common::*;

#[serial]
#[tokio::test]
async fn test_transaction_subscription() {
    let wallet = sender_wallet();
    let sender_address = wallet.public_key.derive_address();
    let destination_address = receiver_address();

    let client = Client::new(server_url());

    let sub_req =
        AccountTransactionsSubscription::proposed([sender_address.clone()])
            .expect("Valid address");

    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, mut stream) =
        conn.subscribe(&sub_req).await.expect("Subscription failed");

    let payment = PaymentBuilder::new(
        sender_address.clone(),
        destination_address.clone(),
        xrp!(7.49),
    )
    .fill(&client)
    .await
    .expect("Failed to auto-fill payment")
    .build()
    .expect("Failed to build payment");

    let submit = SubmitRequestBuilder::new(&payment, &wallet)
        .build()
        .expect("Failed to build submit request");

    client.request(&submit).await.expect("Failed to submit payment");

    let mut validated_found = false;
    while let Ok(msg) = stream.recv().await {
        eprintln!(
            "Received tx [{}]: {}",
            if msg.validated { "validated" } else { "unvalidated" },
            msg.hash
        );

        if msg.validated {
            if let Some(amount) = msg.delivered_amount() {
                eprintln!(
                    "Transaction from {sender_address} to {destination_address}: {amount}"
                );
            }

            assert_eq!(msg.engine_result, "tesSUCCESS");
            validated_found = true;
            break;
        }
    }

    assert!(
        validated_found,
        "Timed out or failed to find a validated transaction"
    );
}

/// Subscribes to one account, submits a payment to prove delivery is live,
/// then unsubscribes/drops and confirms nothing more arrives on the
/// connection. Unlike the global transaction stream, an account-scoped
/// stream is naturally quiet once its own payment is validated, so this
/// silence check is a reliable signal, not just a lucky quiet period.
///
/// Liveness is proved by draining `conn.recv()` (conn's own unified
/// stream) rather than `stream.recv()` - both receive every push
/// independently, so anything drained via `stream` would sit unread in
/// `conn`'s own channel and the silence check would just pick that up.
async fn setup_and_await_own_payment(
    client: &Client,
) -> (
    xrpl::SubscriptionSession,
    xrpl::SubscriptionStream<xrpl::subscriptions::AccountTransactionMessage>,
) {
    let wallet = sender_wallet();
    let sender_address = wallet.public_key.derive_address();
    let destination_address = receiver_address();

    let sub_req =
        AccountTransactionsSubscription::proposed([sender_address.clone()])
            .expect("Valid address");

    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, stream) =
        conn.subscribe(&sub_req).await.expect("Subscription failed");

    let payment = PaymentBuilder::new(
        sender_address.clone(),
        destination_address,
        xrp!(7.49),
    )
    .fill(client)
    .await
    .expect("Failed to auto-fill payment")
    .build()
    .expect("Failed to build payment");

    let submit = SubmitRequestBuilder::new(&payment, &wallet)
        .build()
        .expect("Failed to build submit request");

    client.request(&submit).await.expect("Failed to submit payment");

    while let Ok(event) = conn.recv().await {
        let SubscriptionEvent::Transaction(msg) = event else { continue };
        if msg.validated {
            assert_eq!(msg.engine_result, "tesSUCCESS");
            break;
        }
    }

    (conn, stream)
}

#[serial]
#[tokio::test]
async fn test_account_transaction_subscription_unsubscribe_stops_pushes() {
    let client = Client::new(server_url());
    let (mut conn, stream) = setup_and_await_own_payment(&client).await;

    stream.unsubscribe().await.expect("unsubscribe should succeed");

    let result = timeout(Duration::from_secs(10), conn.recv()).await;
    assert!(
        result.is_err(),
        "no further messages should arrive on this connection after unsubscribing"
    );
}

#[serial]
#[tokio::test]
async fn test_account_transaction_subscription_drop_stops_pushes() {
    let client = Client::new(server_url());
    let (mut conn, stream) = setup_and_await_own_payment(&client).await;

    drop(stream);
    tokio::time::sleep(Duration::from_millis(500)).await;

    let result = timeout(Duration::from_secs(10), conn.recv()).await;
    assert!(
        result.is_err(),
        "no further messages should arrive on this connection after dropping the stream"
    );
}
