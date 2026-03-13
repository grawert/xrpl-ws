mod common;

use xrpl::{Client, request::fee::FeeRequest};
use common::*;

#[ignore]
#[tokio::test]
async fn test_fee_request() {
    let client = Client::new(&server_url());
    let response = client.request(FeeRequest).await.unwrap();
    let result = response.result().unwrap();

    assert!(!result.drops.base_fee.is_empty(), "base_fee should not be empty");
    assert!(
        !result.drops.minimum_fee.is_empty(),
        "minimum_fee should not be empty"
    );
    assert!(
        !result.drops.median_fee.is_empty(),
        "median_fee should not be empty"
    );
    assert!(
        !result.drops.open_ledger_fee.is_empty(),
        "open_ledger_fee should not be empty"
    );
    assert!(
        !result.levels.median_level.is_empty(),
        "median_level should not be empty"
    );
    assert!(
        !result.levels.minimum_level.is_empty(),
        "minimum_level should not be empty"
    );
    assert!(
        !result.levels.open_ledger_level.is_empty(),
        "open_ledger_level should not be empty"
    );
    assert!(result.ledger_current_index > 0, "ledger index should be positive");

    println!("Fee drops: {:?}", result.drops);
    println!("Fee levels: {:?}", result.levels);
    println!("Ledger index: {}", result.ledger_current_index);
}
