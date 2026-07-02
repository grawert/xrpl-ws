mod common;

use serial_test::serial;
use tokio::time::{timeout, Duration};
use xrpl::subscriptions::{BookChangesSubscription, LedgerSubscription};
use xrpl::Client;
use common::*;

/// Two independent subscriptions opened over the same shared connection must
/// each receive only their own message type. If the driver's per-type
/// fan-out (Task 1, step 7 of the multi-stream refactor) were broken and
/// every push landed on a single shared channel instead, one of the two
/// `recv()` calls below would either hang waiting for a message tagged for
/// the other stream, or receive the wrong type and have to discard it.
#[serial]
#[tokio::test]
async fn test_concurrent_subscriptions_over_one_handle_are_isolated() {
    let client = Client::new(server_url());

    let mut handle = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");

    let (_ledger_resp, mut ledger_stream) = handle
        .subscribe(&LedgerSubscription::new())
        .await
        .expect("Ledger subscription failed");

    let (_book_resp, mut book_stream) = handle
        .subscribe(&BookChangesSubscription::default())
        .await
        .expect("BookChanges subscription failed");

    let ledger_msg = timeout(Duration::from_secs(30), ledger_stream.recv())
        .await
        .expect("Timed out waiting for a ledger close")
        .expect("Ledger stream closed unexpectedly");
    assert!(ledger_msg.ledger_index > 0);
    assert!(!ledger_msg.ledger_hash.is_empty());

    let book_msg = timeout(Duration::from_secs(30), book_stream.recv())
        .await
        .expect("Timed out waiting for a book_changes message")
        .expect("BookChanges stream closed unexpectedly");
    assert!(book_msg.ledger_index > 0);
}
