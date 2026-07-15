mod common;

use serial_test::serial;
use tokio::time::{timeout, Duration};
use xrpl::subscriptions::ledger::LedgerSubscription;
use xrpl::Client;
use common::*;

#[serial]
#[tokio::test]
async fn test_ledger_subscription() {
    let client = Client::new(server_url());

    let subscription = LedgerSubscription::default();

    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, mut stream) = conn
        .subscribe(&subscription)
        .await
        .expect("Ledger subscription failed");

    match stream.recv().await {
        Ok(ledger) => {
            assert!(ledger.ledger_index > 0);
            assert!(!ledger.ledger_hash.is_empty());
        }
        Err(e) => panic!("Broadcast receiver error: {e:?}"),
    }
}

// Ledger closes happen roughly every 3-5s on testnet regardless of any
// account activity, so an "expect silence" window after unsubscribing/
// dropping is a reliable signal here - unlike activity-driven streams,
// there is no natural quiet period to confuse the result with.
//
// Liveness is proved via `conn.recv()` (conn's own unified stream)
// rather than `stream.recv()`. Both receive every push independently - if
// we drained the "proof of life" message via `stream` instead, it would sit
// unread in `conn`'s own channel and the silence check below would just
// pick up that stale message, not a genuinely new one.

#[serial]
#[tokio::test]
async fn test_ledger_subscription_unsubscribe_stops_pushes() {
    let client = Client::new(server_url());
    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, stream) = conn
        .subscribe(&LedgerSubscription::new())
        .await
        .expect("Ledger subscription failed");

    conn.recv().await.expect("should receive at least one ledger close");

    stream.unsubscribe().await.expect("unsubscribe should succeed");

    let result = timeout(Duration::from_secs(10), conn.recv()).await;

    assert!(
        result.is_err(),
        "no further messages should arrive on this connection after unsubscribing"
    );
}

#[serial]
#[tokio::test]
async fn test_ledger_subscription_drop_stops_pushes() {
    let client = Client::new(server_url());
    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, stream) = conn
        .subscribe(&LedgerSubscription::new())
        .await
        .expect("Ledger subscription failed");

    conn.recv().await.expect("should receive at least one ledger close");

    drop(stream);
    // Give the driver a moment to process the fire-and-forget unsubscribe
    // sent from `Drop` before we start expecting silence.
    tokio::time::sleep(Duration::from_millis(500)).await;

    let result = timeout(Duration::from_secs(10), conn.recv()).await;

    assert!(
        result.is_err(),
        "no further messages should arrive on this connection after dropping the stream"
    );
}
