mod common;

use xrpl::*;
use common::*;
use serde_json::json;

#[ignore]
#[tokio::test]
async fn test_account_currencies() {
    let client = XrplClient::new(SERVER_URL).await.unwrap();
    let request = request::account_currencies::AccountCurrenciesRequest {
        account: TEST_ACCOUNT.to_string(),
        ledger_index: Some(json!("validated")),
        ..Default::default()
    };

    let response = client.request(request).await.unwrap();
    let result =
        response.result().expect("Expected account currencies in response");

    assert!(!result.send_currencies.is_empty());
    assert!(!result.receive_currencies.is_empty());
}
