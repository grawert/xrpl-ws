mod common;

use xrpl::request::amm_info::AmmInfoRequest;
use xrpl::request::XrplRequest;
use xrpl::types::Asset;
use xrpl::{Client, XrplError};
use common::*;

#[tokio::test]
async fn test_amm_info_by_assets() {
    let client = Client::new(server_url());

    let request = AmmInfoRequest {
        asset: Some(Asset::xrp()),
        asset2: Some(
            Asset::token("TST", "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd").unwrap(),
        ),
        ledger_index: Some("validated".into()),
        ..Default::default()
    };

    // Accept any application-level error (e.g. pool not on this testnet instance).
    // Only transport failures (Disconnected, Timeout) are unexpected.
    match client.request(request).await {
        Ok(_) | Err(XrplError::ApiError { .. }) => {}
        Err(e) => panic!("unexpected transport error: {e}"),
    }
}

#[tokio::test]
async fn test_amm_info_by_account() {
    let client = Client::new(server_url());

    let request = AmmInfoRequest {
        amm_account: Some("rp9E3FN3gNmvePGhYnf414T2TkUuoxu8vM".to_string()),
        account: Some("rQhWct2fv4Vc4KRjRgMrxa8xPN9Zx9iLKV".to_string()),
        ledger_index: Some("current".into()),
        ..Default::default()
    };

    match client.request(request).await {
        Ok(_) | Err(XrplError::ApiError { .. }) => {}
        Err(e) => panic!("unexpected transport error: {e}"),
    }
}

#[test]
fn test_amm_info_serialization() {
    // Test serialization by assets
    let request = AmmInfoRequest {
        asset: Some(Asset::xrp()),
        asset2: Some(
            Asset::token("TST", "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd").unwrap(),
        ),
        ledger_index: Some("validated".into()),
        ..Default::default()
    };

    let json = request.to_value();
    assert_eq!(json["command"], "amm_info");
    assert!(json["asset"].is_object());
    assert_eq!(json["asset"]["currency"], "XRP");
    assert!(json["asset2"].is_object());
    assert_eq!(json["asset2"]["currency"], "TST");
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
