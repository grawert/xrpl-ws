mod common;

use xrpl::*;
use common::*;
use serde_json::json;

#[ignore]
#[tokio::test]
async fn test_account_channels() {
    let client = Client::new(&server_url());
    let request = request::account_channels::AccountChannelsRequest {
        account: sender_address(),
        ledger_index: Some(json!("validated")),
        ..Default::default()
    };

    let response = client.request(request).await.unwrap();
    let result =
        response.result().expect("Expected account channels in response");

    assert_eq!(result.account, sender_address());
}
