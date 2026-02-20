mod common;

use xrpl::*;
use common::*;
use serde_json::json;

const NFT_TEST_ACCOUNT: &str = "raQshXKbbqYaQUcanRkwusEyV5eJdW9KpR";

#[ignore]
#[tokio::test]
async fn test_account_nfts() {
    let client = XrplClient::new(SERVER_URL).await.unwrap();
    let request = request::account_nfts::AccountNftsRequest {
        account: NFT_TEST_ACCOUNT.to_string(),
        ledger_index: Some(json!("validated")),
        ..Default::default()
    };

    let response = client.request(request).await.unwrap();
    let result = response.result().expect("Expected account nfts in response");

    assert!(!result.account_nfts.is_empty());
}
