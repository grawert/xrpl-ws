mod common;

use xrpl::*;
use common::*;
use serde_json::json;

#[ignore]
#[tokio::test]
async fn test_account_channels() {
    let client = XrplClient::new(SERVER_URL).await.unwrap();
    let request = request::account_channels::AccountChannelsRequest {
        account: TEST_ACCOUNT.to_string(),
        ledger_index: Some(json!("validated")),
        ..Default::default()
    };

    let response = client.request(request).await.unwrap();
    let result =
        response.result().expect("Expected account channels in response");

    assert_eq!(result.account, TEST_ACCOUNT);
}
