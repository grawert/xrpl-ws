mod common;

use xrpl::Client;
use xrpl::request::ledger_entry::LedgerEntryRequest;
use xrpl::request::XrplRequest;
use xrpl::types::Asset;
use common::*;

#[tokio::test]
async fn test_ledger_entry_account_root() {
    let client = Client::new(server_url());
    let request = LedgerEntryRequest::for_account_root(sender_address())
        .with_ledger_index("validated");
    let result = client
        .request(&request)
        .await
        .expect("LedgerEntry request (account_root) failed")
        .result()
        .expect("Could not get result from LedgerEntry response");

    assert_eq!(result.index.len(), 64);
    let node = result.node.expect("node should be present");
    assert_eq!(node["LedgerEntryType"], "AccountRoot");
    assert_eq!(node["Account"], sender_address());
}

#[tokio::test]
async fn test_ledger_entry_by_index() {
    let client = Client::new(server_url());

    // First get the account root index via account_root lookup
    let first_request = LedgerEntryRequest::for_account_root(sender_address())
        .with_ledger_index("validated");
    let first = client
        .request(&first_request)
        .await
        .expect("LedgerEntry request (account_root) failed")
        .result()
        .expect("Could not get result from LedgerEntry response");

    let index = first.index.clone();

    // Now look up the same entry by its raw index
    let request = LedgerEntryRequest::by_index(index.clone())
        .with_ledger_index("validated");
    let result = client
        .request(&request)
        .await
        .expect("LedgerEntry request (by index) failed")
        .result()
        .expect("Could not get result from LedgerEntry response");

    assert_eq!(result.index, index);
    let node = result.node.expect("node should be present");
    assert_eq!(node["LedgerEntryType"], "AccountRoot");
}

// Serialization tests - no network required

#[test]
fn test_ledger_entry_account_root_serializes() {
    let req = LedgerEntryRequest::for_account_root(
        "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
    )
    .with_ledger_index("validated");
    let json = req.to_value().expect("Failed to serialize request");
    assert_eq!(json["command"], "ledger_entry");
    assert_eq!(json["account_root"], "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    assert_eq!(json["ledger_index"], "validated");
    assert!(json["index"].is_null());
    assert!(json["escrow"].is_null());
}

#[test]
fn test_ledger_entry_escrow_key_serializes() {
    let req = LedgerEntryRequest::for_escrow(
        "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        42,
    );
    let json = req.to_value().expect("Failed to serialize request");
    assert_eq!(json["escrow"]["owner"], "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    assert_eq!(json["escrow"]["seq"], 42);
}

#[test]
fn test_ledger_entry_offer_key_serializes() {
    let req =
        LedgerEntryRequest::for_offer("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", 7);
    let json = req.to_value().expect("Failed to serialize request");
    assert_eq!(json["offer"]["account"], "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    assert_eq!(json["offer"]["seq"], 7);
}

#[test]
fn test_ledger_entry_ripple_state_key_serializes() {
    let req = LedgerEntryRequest::for_ripple_state(
        [
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        ],
        "USD",
    );
    let json = req.to_value().expect("Failed to serialize request");
    assert_eq!(json["ripple_state"]["currency"], "USD");
    assert!(json["ripple_state"]["accounts"].is_array());
}

#[test]
fn test_ledger_entry_amm_key_serializes() {
    let req = LedgerEntryRequest::for_amm(
        Asset::xrp(),
        Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
            .expect("Failed to create USD asset"),
    );
    let json = req.to_value().expect("Failed to serialize request");
    assert!(json["amm"]["asset"].is_object());
    assert!(json["amm"]["asset2"].is_object());
    assert_eq!(json["amm"]["asset2"]["currency"], "USD");
}
