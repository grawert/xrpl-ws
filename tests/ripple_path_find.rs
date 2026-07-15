mod common;

use xrpl::Client;
use xrpl::drops;
use xrpl::request::ripple_path_find::RipplePathFindRequest;
use xrpl::request::XrplRequest;
use xrpl::types::Amount;
use common::*;

#[tokio::test]
async fn test_ripple_path_find_xrp() {
    let client = Client::new(server_url());

    // Path find for a direct XRP payment - should always find a trivial path
    let request = RipplePathFindRequest::new(
        sender_address(),
        receiver_address(),
        drops!(1_000_000), // 1 XRP
    )
    .with_ledger_index("validated");
    let result = client
        .request(&request)
        .await
        .expect("ripple_path_find (XRP) request failed")
        .result()
        .expect("Could not get result from ripple_path_find response");

    assert_eq!(result.source_account, sender_address());
    assert_eq!(result.destination_account, receiver_address());
    // Direct XRP payments need no intermediate path - alternatives is empty
    // but the response must deserialize cleanly and full_reply should be set
    assert!(result.full_reply.unwrap_or(false));
}

#[tokio::test]
async fn test_ripple_path_find_issued_currency() {
    let client = Client::new(server_url());

    // Path find for an issued currency payment - alternatives may be empty
    // if no path exists, which is fine; we just verify the response deserializes
    let request = RipplePathFindRequest::new(
        sender_address(),
        receiver_address(),
        Amount::issued_currency(
            "1",
            "USD",
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        )
        .expect("Failed to create issued currency amount"),
    )
    .with_ledger_index("validated");
    let _ = client
        .request(&request)
        .await
        .expect("ripple_path_find (IOU) request failed")
        .result()
        .expect("Could not get result from ripple_path_find response");
}

#[test]
fn test_ripple_path_find_serializes() {
    let req = RipplePathFindRequest::new("rSource", "rDest", drops!(1_000_000))
        .with_ledger_index("validated");
    let json = req.to_value().expect("Failed to serialize request");
    assert_eq!(json["command"], "ripple_path_find");
    assert_eq!(json["source_account"], "rSource");
    assert_eq!(json["destination_account"], "rDest");
    assert!(json["destination_amount"].is_string()); // XRP is a string amount
    assert_eq!(json["ledger_index"], "validated");
    assert!(json["send_max"].is_null());
    assert!(json["source_currencies"].is_null());
}
