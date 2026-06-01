mod common;

use xrpl::*;
use common::*;
use serde_json::json;

#[tokio::test]
async fn test_account_currencies() {
    let client = Client::new(server_url());
    let request = request::account_currencies::AccountCurrenciesRequest {
        account: sender_address(),
        ledger_index: Some(json!("validated")),
        ..Default::default()
    };

    let response = client.request(request).await.unwrap();
    let result =
        response.result().expect("Expected account currencies in response");

    // A validated-ledger response always sets this flag.
    assert!(
        result.validated.unwrap(),
        "response should be from a validated ledger"
    );
    // Fresh accounts may not have any trust lines, so the currency arrays
    // can legitimately be empty — we only verify the types are correct.
    for code in result.send_currencies.iter().chain(&result.receive_currencies)
    {
        assert!(
            code.len() == 3 || code.len() == 40,
            "unexpected currency code format: {code}"
        );
    }
}
