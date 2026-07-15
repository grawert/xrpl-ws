mod common;

use serial_test::serial;
use tokio::time::{timeout, Duration};
use xrpl::subscriptions::BookChangesSubscription;
use xrpl::Client;
use common::*;

#[serial]
#[tokio::test]
async fn test_book_changes_subscription() {
    let client = Client::new(server_url());

    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, mut stream) = conn
        .subscribe(&BookChangesSubscription::default())
        .await
        .expect("BookChanges subscription failed");

    let msg = timeout(Duration::from_secs(30), stream.recv())
        .await
        .expect("Timed out waiting for a book_changes message")
        .expect("BookChanges stream closed unexpectedly");
    assert!(msg.ledger_index > 0);
}

#[serial]
#[tokio::test]
async fn test_book_changes_subscription_unsubscribe_is_rejected_by_server() {
    let client = Client::new(server_url());
    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, stream) = conn
        .subscribe(&BookChangesSubscription::default())
        .await
        .expect("BookChanges subscription failed");

    conn.recv()
        .await
        .expect("should receive at least one book_changes message");

    let err = stream
        .unsubscribe()
        .await
        .expect_err("rippled is known to reject unsubscribe for book_changes");

    match err {
        xrpl::XrplError::ApiError { error, .. } => {
            assert_eq!(error, "malformedStream");
        }
        other => panic!("expected an ApiError, got {other:?}"),
    }
}

#[serial]
#[tokio::test]
async fn test_book_changes_subscription_drop_does_not_break_connection() {
    let client = Client::new(server_url());
    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, stream) = conn
        .subscribe(&BookChangesSubscription::default())
        .await
        .expect("BookChanges subscription failed");

    conn.recv()
        .await
        .expect("should receive at least one book_changes message");

    drop(stream);
    // Give the driver a moment to process the fire-and-forget unsubscribe
    // sent from `Drop`. Rippled will reject it (see the test above), but
    // `Drop` can't observe that - it's fire-and-forget by design.
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Since the server never actually removed the subscription, the
    // connection keeps delivering - this just confirms the drop path
    // didn't otherwise corrupt the connection.
    timeout(Duration::from_secs(10), conn.recv())
        .await
        .expect("timed out waiting for a message")
        .expect("connection should still be usable after the drop");
}
