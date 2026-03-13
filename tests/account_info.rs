mod common;

use xrpl::*;
use xrpl::request::account_info::AccountInfoRequest;
use common::*;

#[ignore]
#[tokio::test]
async fn test_account_info() {
    let client = Client::new(&server_url());
    let request =
        AccountInfoRequest { account: sender_address(), ..Default::default() };

    let response = client.request(request).await.unwrap();
    let result = response.result().expect("Expected account data in response");
    let account_root = &result.account_data;

    assert_eq!(account_root.account, sender_address());
    assert!(!account_root.balance.is_empty());
    assert!(account_root.sequence > 0);
}
