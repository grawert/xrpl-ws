mod common;

use xrpl::request::amm_info::AmmInfoRequest;
use xrpl::request::XrplRequest;
use xrpl::types::Asset;
use xrpl::{Client, XrplError};
use common::*;

#[tokio::test]
async fn test_amm_info_by_assets() {
    let client = Client::new(server_url());

    let request = AmmInfoRequest::by_assets(
        Asset::xrp(),
        Asset::token("TST", "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd")
            .expect("Failed to create TST asset"),
    )
    .with_ledger_index("validated");

    // Accept any application-level error (e.g. pool not on this testnet instance).
    // Only transport failures (Disconnected, Timeout) are unexpected.
    match client.request(&request).await {
        Ok(_) | Err(XrplError::ApiError { .. }) => {}
        Err(e) => panic!("unexpected transport error: {e}"),
    }
}

#[tokio::test]
async fn test_amm_info_by_account() {
    let client = Client::new(server_url());

    let request =
        AmmInfoRequest::by_account("rp9E3FN3gNmvePGhYnf414T2TkUuoxu8vM")
            .with_account("rQhWct2fv4Vc4KRjRgMrxa8xPN9Zx9iLKV")
            .with_ledger_index("current");

    match client.request(&request).await {
        Ok(_) | Err(XrplError::ApiError { .. }) => {}
        Err(e) => panic!("unexpected transport error: {e}"),
    }
}

#[test]
fn test_amm_info_serialization() {
    // Test serialization by assets
    let request = AmmInfoRequest::by_assets(
        Asset::xrp(),
        Asset::token("TST", "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd")
            .expect("Failed to create TST asset"),
    )
    .with_ledger_index("validated");

    let json = request.to_value().expect("Failed to serialize request");
    assert_eq!(json["command"], "amm_info");
    assert!(json["asset"].is_object());
    assert_eq!(json["asset"]["currency"], "XRP");
    assert!(json["asset2"].is_object());
    assert_eq!(json["asset2"]["currency"], "TST");
    assert_eq!(json["ledger_index"], "validated");
    assert!(json["amm_account"].is_null());

    // Test serialization by AMM account
    let request =
        AmmInfoRequest::by_account("rp9E3FN3gNmvePGhYnf414T2TkUuoxu8vM")
            .with_account("rQhWct2fv4Vc4KRjRgMrxa8xPN9Zx9iLKV");

    let json = request.to_value().expect("Failed to serialize request");
    assert_eq!(json["command"], "amm_info");
    assert_eq!(json["amm_account"], "rp9E3FN3gNmvePGhYnf414T2TkUuoxu8vM");
    assert_eq!(json["account"], "rQhWct2fv4Vc4KRjRgMrxa8xPN9Zx9iLKV");
    assert!(json["asset"].is_null());
    assert!(json["asset2"].is_null());
}
