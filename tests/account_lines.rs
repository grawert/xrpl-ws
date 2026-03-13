mod common;

use xrpl::*;
use common::*;
use serde_json::json;

#[ignore]
#[tokio::test]
async fn test_account_lines() {
    let client = Client::new(&server_url());
    let request = request::account_lines::AccountLinesRequest {
        account: sender_address(),
        ledger_index: Some(json!("validated")),
        ..Default::default()
    };

    let response = client.request(request).await.unwrap();
    let result = response.result().expect("Expected account lines in response");

    // Fresh accounts may not have any trust lines established
    // Just verify the response structure is correct and request succeeded
    println!("Account lines count: {}", result.lines.len());
}
