mod common;

use xrpl::*;
use common::*;

fn nft_account() -> String {
    std::env::var("TEST_NFT_ACCOUNT")
        .unwrap_or_else(|_| "raQshXKbbqYaQUcanRkwusEyV5eJdW9KpR".to_string())
}

#[tokio::test]
async fn test_account_nfts() {
    let client = Client::new(server_url());
    let account = nft_account();
    let request =
        request::account_nfts::AccountNftsRequest::new(account.clone())
            .with_ledger_index("validated");

    let response =
        client.request(&request).await.expect("Failed to request account NFTs");
    let result = response.result().expect("Expected account nfts in response");

    assert!(
        !result.account_nfts.is_empty(),
        "account {account} should have at least one NFT - set TEST_NFT_ACCOUNT to a funded testnet account"
    );
}
