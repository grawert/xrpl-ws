mod common;

use xrpl::*;
use common::*;
use serde_json::json;

#[ignore]
#[tokio::test]
async fn test_account_currencies() {
    let client = Client::new(&server_url());
    let request = request::account_currencies::AccountCurrenciesRequest {
        account: sender_address(),
        ledger_index: Some(json!("validated")),
        ..Default::default()
    };

    let response = client.request(request).await.unwrap();
    let result =
        response.result().expect("Expected account currencies in response");

    // Fresh accounts may not have any trust lines, so currencies arrays could be empty
    // Just verify the response structure is correct and request succeeded
    println!("Send currencies count: {}", result.send_currencies.len());
    println!("Receive currencies count: {}", result.receive_currencies.len());
}
