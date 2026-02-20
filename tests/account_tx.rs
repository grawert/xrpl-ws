mod common;

use xrpl::*;
use xrpl::request::account_tx::AccountTxRequest;
use common::*;

const DEFAULT_TX_LIMIT: u32 = 3;

#[ignore]
#[tokio::test]
async fn test_account_tx() {
    let client = XrplClient::new(SERVER_URL).await.unwrap();
    let request = AccountTxRequest {
        account: TEST_ACCOUNT.to_string(),
        limit: Some(DEFAULT_TX_LIMIT),
        ..Default::default()
    };

    let response = client.request(request).await.unwrap();
    let result = response.result().expect("Expected transactions in response");

    assert!(!result.transactions.is_empty());
}
