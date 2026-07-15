mod common;

use xrpl::*;
use xrpl::request::account_info::AccountInfoRequest;
use common::*;

#[tokio::test]
async fn test_account_info() {
    let client = Client::new(server_url());
    let request = AccountInfoRequest::new(sender_address());

    let response =
        client.request(&request).await.expect("Failed to request account info");
    let result = response.result().expect("Expected account data in response");
    let account_root = &result.account_data;

    assert_eq!(account_root.account, sender_address());
    assert!(!account_root.balance.is_empty());
    assert!(account_root.sequence > 0);
}
