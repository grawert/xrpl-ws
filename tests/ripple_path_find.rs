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

    // Path find for a direct XRP payment — should always find a trivial path
    let result = client
        .request(RipplePathFindRequest {
            source_account: sender_address(),
            destination_account: receiver_address(),
            destination_amount: drops!(1_000_000), // 1 XRP
            ledger_index: Some("validated".into()),
            ..Default::default()
        })
        .await
        .unwrap()
        .result()
        .unwrap();

    assert_eq!(result.source_account, sender_address());
    assert_eq!(result.destination_account, receiver_address());
    // Direct XRP payments need no intermediate path — alternatives is empty
    // but the response must deserialize cleanly and full_reply should be set
    assert!(result.full_reply.unwrap_or(false));
}

#[tokio::test]
async fn test_ripple_path_find_issued_currency() {
    let client = Client::new(server_url());

    // Path find for an issued currency payment — alternatives may be empty
    // if no path exists, which is fine; we just verify the response deserializes
    let _ = client
        .request(RipplePathFindRequest {
            source_account: sender_address(),
            destination_account: receiver_address(),
            destination_amount: Amount::issued_currency(
                "1",
                "USD",
                "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            )
            .unwrap(),
            ledger_index: Some("validated".into()),
            ..Default::default()
        })
        .await
        .unwrap()
        .result()
        .unwrap();
}

#[test]
fn test_ripple_path_find_serializes() {
    let req = RipplePathFindRequest {
        source_account: "rSource".to_string(),
        destination_account: "rDest".to_string(),
        destination_amount: drops!(1_000_000),
        ledger_index: Some("validated".into()),
        ..Default::default()
    };
    let json = req.to_value();
    assert_eq!(json["command"], "ripple_path_find");
    assert_eq!(json["source_account"], "rSource");
    assert_eq!(json["destination_account"], "rDest");
    assert!(json["destination_amount"].is_string()); // XRP is a string amount
    assert_eq!(json["ledger_index"], "validated");
    assert!(json["send_max"].is_null());
    assert!(json["source_currencies"].is_null());
}
