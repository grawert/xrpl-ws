mod common;

use xrpl::Client;
use xrpl::request::account_tx::AccountTxRequest;
use common::*;

const DEFAULT_TX_LIMIT: u32 = 3;

#[ignore]
#[tokio::test]
async fn test_account_tx() {
    let client = Client::new(&server_url());
    let request = AccountTxRequest {
        account: sender_address(),
        limit: Some(DEFAULT_TX_LIMIT),
        ..Default::default()
    };

    let response = client.request(request).await.unwrap();
    let result = response.result().expect("Expected transactions in response");

    assert!(!result.transactions.is_empty());
}
