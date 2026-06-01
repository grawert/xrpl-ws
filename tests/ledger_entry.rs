mod common;

use xrpl::Client;
use xrpl::request::ledger_entry::{
    AmmLedgerKey, EscrowLedgerKey, LedgerEntryRequest, OfferLedgerKey,
    RippleStateLedgerKey,
};
use xrpl::request::XrplRequest;
use xrpl::types::Asset;
use common::*;

#[tokio::test]
async fn test_ledger_entry_account_root() {
    let client = Client::new(server_url());
    let result = client
        .request(LedgerEntryRequest {
            account_root: Some(sender_address()),
            ledger_index: Some("validated".into()),
            ..Default::default()
        })
        .await
        .unwrap()
        .result()
        .unwrap();

    assert_eq!(result.index.len(), 64);
    let node = result.node.expect("node should be present");
    assert_eq!(node["LedgerEntryType"], "AccountRoot");
    assert_eq!(node["Account"], sender_address());
}

#[tokio::test]
async fn test_ledger_entry_by_index() {
    let client = Client::new(server_url());

    // First get the account root index via account_root lookup
    let first = client
        .request(LedgerEntryRequest {
            account_root: Some(sender_address()),
            ledger_index: Some("validated".into()),
            ..Default::default()
        })
        .await
        .unwrap()
        .result()
        .unwrap();

    let index = first.index.clone();

    // Now look up the same entry by its raw index
    let result = client
        .request(LedgerEntryRequest {
            index: Some(index.clone()),
            ledger_index: Some("validated".into()),
            ..Default::default()
        })
        .await
        .unwrap()
        .result()
        .unwrap();

    assert_eq!(result.index, index);
    let node = result.node.expect("node should be present");
    assert_eq!(node["LedgerEntryType"], "AccountRoot");
}

// Serialization tests — no network required

#[test]
fn test_ledger_entry_account_root_serializes() {
    let req = LedgerEntryRequest {
        account_root: Some("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string()),
        ledger_index: Some("validated".into()),
        ..Default::default()
    };
    let json = req.to_value();
    assert_eq!(json["command"], "ledger_entry");
    assert_eq!(json["account_root"], "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    assert_eq!(json["ledger_index"], "validated");
    assert!(json["index"].is_null());
    assert!(json["escrow"].is_null());
}

#[test]
fn test_ledger_entry_escrow_key_serializes() {
    let req = LedgerEntryRequest {
        escrow: Some(EscrowLedgerKey {
            owner: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
            seq: 42,
        }),
        ..Default::default()
    };
    let json = req.to_value();
    assert_eq!(json["escrow"]["owner"], "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    assert_eq!(json["escrow"]["seq"], 42);
}

#[test]
fn test_ledger_entry_offer_key_serializes() {
    let req = LedgerEntryRequest {
        offer: Some(OfferLedgerKey {
            account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
            seq: 7,
        }),
        ..Default::default()
    };
    let json = req.to_value();
    assert_eq!(json["offer"]["account"], "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    assert_eq!(json["offer"]["seq"], 7);
}

#[test]
fn test_ledger_entry_ripple_state_key_serializes() {
    let req = LedgerEntryRequest {
        ripple_state: Some(RippleStateLedgerKey {
            accounts: [
                "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
                "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
            ],
            currency: "USD".to_string(),
        }),
        ..Default::default()
    };
    let json = req.to_value();
    assert_eq!(json["ripple_state"]["currency"], "USD");
    assert!(json["ripple_state"]["accounts"].is_array());
}

#[test]
fn test_ledger_entry_amm_key_serializes() {
    let req = LedgerEntryRequest {
        amm: Some(AmmLedgerKey {
            asset: Asset::xrp(),
            asset2: Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
                .unwrap(),
        }),
        ..Default::default()
    };
    let json = req.to_value();
    assert!(json["amm"]["asset"].is_object());
    assert!(json["amm"]["asset2"].is_object());
    assert_eq!(json["amm"]["asset2"]["currency"], "USD");
}
