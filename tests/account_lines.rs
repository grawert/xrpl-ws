mod common;

use xrpl::*;
use common::*;
use serde_json::json;

#[tokio::test]
async fn test_account_lines() {
    let client = Client::new(server_url());
    let request = request::account_lines::AccountLinesRequest {
        account: sender_address(),
        ledger_index: Some(json!("validated")),
        ..Default::default()
    };

    let response = client.request(request).await.unwrap();
    let result = response.result().expect("Expected account lines in response");

    assert_eq!(result.account, sender_address());
    // Fresh accounts may have no trust lines; verify each returned line is well-formed.
    for line in &result.lines {
        assert!(
            !line.account.is_empty(),
            "trust line account should not be empty"
        );
        assert!(
            !line.currency.is_empty(),
            "trust line currency should not be empty"
        );
    }
}
