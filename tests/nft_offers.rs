mod common;

use xrpl::Client;
use xrpl::request::account_nfts::AccountNftsRequest;
use xrpl::request::nft_buy_offers::NftBuyOffersRequest;
use xrpl::request::nft_sell_offers::NftSellOffersRequest;
use xrpl::request::XrplRequest;
use common::*;

/// Fetches the first NFT owned by the test account, if any.
async fn first_owned_nft(client: &xrpl::Client) -> Option<String> {
    let result = client
        .request(AccountNftsRequest {
            account: sender_address(),
            limit: Some(1),
            ..Default::default()
        })
        .await
        .ok()?
        .result()
        .ok()?;

    result.account_nfts.into_iter().next().map(|n| n.nftoken_id)
}

#[tokio::test]
async fn test_nft_buy_offers() {
    let client = Client::new(server_url());

    let Some(nft_id) = first_owned_nft(&client).await else {
        // No NFTs on the test account — skip rather than fail
        return;
    };

    let result = client
        .request(NftBuyOffersRequest {
            nft_id: nft_id.clone(),
            limit: Some(10),
            ..Default::default()
        })
        .await
        .unwrap()
        .result()
        .unwrap();

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

    let result = client
        .request(NftSellOffersRequest {
            nft_id: nft_id.clone(),
            limit: Some(10),
            ..Default::default()
        })
        .await
        .unwrap()
        .result()
        .unwrap();

    assert_eq!(result.nft_id, nft_id);
    for offer in &result.offers {
        assert!(!offer.nft_offer_index.is_empty());
        assert!(!offer.owner.is_empty());
    }
}

#[test]
fn test_nft_buy_offers_serializes() {
    let req = NftBuyOffersRequest {
        nft_id:
            "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65"
                .to_string(),
        limit: Some(20),
        ..Default::default()
    };
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
    let req = NftSellOffersRequest {
        nft_id:
            "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65"
                .to_string(),
        limit: Some(20),
        ..Default::default()
    };
    let json = req.to_value();
    assert_eq!(json["command"], "nft_sell_offers");
    assert_eq!(json["limit"], 20);
}
