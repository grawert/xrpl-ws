mod common;

use xrpl::*;
use common::*;

#[tokio::test]
async fn test_account_channels() {
    let client = Client::new(server_url());
    let request = request::account_channels::AccountChannelsRequest::new(
        sender_address(),
    )
    .with_ledger_index("validated");

    let response = client
        .request(&request)
        .await
        .expect("Failed to request account channels");
    let result =
        response.result().expect("Expected account channels in response");

    assert_eq!(result.account, sender_address());
}
