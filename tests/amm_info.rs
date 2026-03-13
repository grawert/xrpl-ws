mod common;

use xrpl::*;
use xrpl::request::amm_info::AmmInfoRequest;
use xrpl::request::XrplRequest;
use xrpl::types::Amount;
use common::*;

#[ignore]
#[tokio::test]
async fn test_amm_info_by_assets() {
    let client = Client::new(&server_url());

    // Create AMM info request using asset pair
    let asset = Amount::drops("100000000").unwrap(); // 100 XRP in drops
    let asset2 = Amount::issued_currency(
        "25.81656470648473",
        "TST",
        "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd",
    )
    .unwrap();

    let request = AmmInfoRequest {
        asset: Some(asset),
        asset2: Some(asset2),
        ledger_index: Some("validated".into()),
        ..Default::default()
    };

    let _ = client.request(request).await;
    // Note: We don't assert on specific response content since AMMs may not exist on testnet
}

#[ignore]
#[tokio::test]
async fn test_amm_info_by_account() {
    let client = Client::new(&server_url());

    // Create AMM info request using AMM account
    let request = AmmInfoRequest {
        amm_account: Some("rp9E3FN3gNmvePGhYnf414T2TkUuoxu8vM".to_string()),
        account: Some("rQhWct2fv4Vc4KRjRgMrxa8xPN9Zx9iLKV".to_string()),
        ledger_index: Some("current".into()),
        ..Default::default()
    };

    let _ = client.request(request).await;
    // Note: We don't assert on specific response content since AMMs may not exist on testnet
}

#[test]
fn test_amm_info_serialization() {
    // Test serialization by assets
    let asset = Amount::drops("100000000").unwrap(); // 100 XRP in drops
    let asset2 = Amount::issued_currency(
        "25.0",
        "TST",
        "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd",
    )
    .unwrap();

    let request = AmmInfoRequest {
        asset: Some(asset),
        asset2: Some(asset2),
        ledger_index: Some("validated".into()),
        ..Default::default()
    };

    let json = request.to_value();
    assert_eq!(json["command"], "amm_info");
    assert!(json["asset"].is_object() || json["asset"].is_string());
    assert!(json["asset2"].is_object());
    assert_eq!(json["ledger_index"], "validated");
    assert!(json["amm_account"].is_null());

    // Test serialization by AMM account
    let request = AmmInfoRequest {
        amm_account: Some("rp9E3FN3gNmvePGhYnf414T2TkUuoxu8vM".to_string()),
        account: Some("rQhWct2fv4Vc4KRjRgMrxa8xPN9Zx9iLKV".to_string()),
        ..Default::default()
    };

    let json = request.to_value();
    assert_eq!(json["command"], "amm_info");
    assert_eq!(json["amm_account"], "rp9E3FN3gNmvePGhYnf414T2TkUuoxu8vM");
    assert_eq!(json["account"], "rQhWct2fv4Vc4KRjRgMrxa8xPN9Zx9iLKV");
    assert!(json["asset"].is_null());
    assert!(json["asset2"].is_null());
}
