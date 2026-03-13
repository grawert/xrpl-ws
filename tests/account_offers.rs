mod common;

use xrpl::*;
use common::*;
use serde_json::json;

#[ignore]
#[tokio::test]
async fn test_account_offers() {
    let client = Client::new(&server_url());
    let request = request::account_offers::AccountOffersRequest {
        account: sender_address(),
        ledger_index: Some(json!("validated")),
        ..Default::default()
    };

    let response = client.request(request).await.unwrap();
    let result =
        response.result().expect("Expected account offers in response");

    assert_eq!(result.account, sender_address());
}
