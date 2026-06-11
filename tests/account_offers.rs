mod common;

use xrpl::*;
use common::*;

#[tokio::test]
async fn test_account_offers() {
    let client = Client::new(server_url());
    let request =
        request::account_offers::AccountOffersRequest::new(sender_address())
            .with_ledger_index("validated");

    let response = client
        .request(&request)
        .await
        .expect("Failed to request account offers");
    let result =
        response.result().expect("Expected account offers in response");

    assert_eq!(result.account, sender_address());
}
