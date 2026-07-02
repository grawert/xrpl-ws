mod common;

use xrpl::Client;
use xrpl::request::account_nfts::AccountNftsRequest;
use xrpl::request::nft_buy_offers::NftBuyOffersRequest;
use xrpl::request::nft_sell_offers::NftSellOffersRequest;
use xrpl::request::XrplRequest;
use common::*;

/// Fetches the first NFT owned by the test account, if any.
async fn first_owned_nft(client: &xrpl::Client) -> Option<String> {
    let request = AccountNftsRequest::new(sender_address()).with_limit(1);
    let result = client.request(&request).await.ok()?.result().ok()?;

    result.account_nfts.into_iter().next().map(|n| n.nftoken_id)
}

#[tokio::test]
async fn test_nft_buy_offers() {
    let client = Client::new(server_url());

    let Some(nft_id) = first_owned_nft(&client).await else {
        // No NFTs on the test account — skip rather than fail
        return;
    };

    let request = NftBuyOffersRequest::new(nft_id.clone()).with_limit(10);
    let result = client
        .request(&request)
        .await
        .expect("Failed to request nft_buy_offers")
        .result()
        .expect("Failed to get nft_buy_offers result");

    assert_eq!(result.nft_id, nft_id);
    for offer in &result.offers {
        assert!(!offer.nft_offer_index.is_empty());
        assert!(!offer.owner.is_empty());
    }
}

#[tokio::test]
async fn test_nft_sell_offers() {
    let client = Client::new(server_url());

    let Some(nft_id) = first_owned_nft(&client).await else {
        return;
    };

    let request = NftSellOffersRequest::new(nft_id.clone()).with_limit(10);
    let result = client
        .request(&request)
        .await
        .expect("Failed to request nft_sell_offers")
        .result()
        .expect("Failed to get nft_sell_offers result");

    assert_eq!(result.nft_id, nft_id);
    for offer in &result.offers {
        assert!(!offer.nft_offer_index.is_empty());
        assert!(!offer.owner.is_empty());
    }
}

#[test]
fn test_nft_buy_offers_serializes() {
    let req = NftBuyOffersRequest::new(
        "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65",
    )
    .with_limit(20);
    let json = req.to_value();
    assert_eq!(json["command"], "nft_buy_offers");
    assert_eq!(
        json["nft_id"],
        "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65"
    );
    assert_eq!(json["limit"], 20);
    assert!(json["marker"].is_null());
}

#[test]
fn test_nft_sell_offers_serializes() {
    let req = NftSellOffersRequest::new(
        "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65",
    )
    .with_limit(20);
    let json = req.to_value();
    assert_eq!(json["command"], "nft_sell_offers");
    assert_eq!(json["limit"], 20);
}
