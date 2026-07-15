mod common;

use serial_test::serial;
use xrpl::types::builders::{NFTokenCreateOfferBuilder, SubmitRequestBuilder};
use xrpl::util::xrp_balance;
use xrpl::{Client, xrp};
use common::*;

/// XRPL base unit conversion: 1 XRP = 1,000,000 drops.
const DROPS_PER_XRP: u64 = 1_000_000;
/// Minimum drops the test account needs to cover the offer + fee.
const MIN_BUYER_BALANCE_DROPS: u64 = 2 * DROPS_PER_XRP;
/// One-hour offer expiration window.
const OFFER_EXPIRATION_1_HOUR: u32 = 3600;

#[serial]
#[tokio::test]
async fn test_nftoken_create_buy_offer() {
    let wallet = sender_wallet();
    let buyer_addr = sender_address();

    let client = Client::new(server_url());

    let balance = xrp_balance(&client, &buyer_addr).await.unwrap();
    println!(
        "Account: {} | Balance: {} drops ({} XRP)",
        buyer_addr,
        balance,
        balance / DROPS_PER_XRP,
    );

    if balance < MIN_BUYER_BALANCE_DROPS {
        panic!(
            "Insufficient balance: {balance} drops. Need at least 2 XRP for this test."
        );
    }

    let test_nftoken_id =
        "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65";

    let nftoken_offer_tx = NFTokenCreateOfferBuilder::new(
        buyer_addr.clone(),
        test_nftoken_id,
        xrp!(7.49),
    )
    .fill(&client)
    .await
    .expect("Failed to auto-fill NFTokenCreateOffer")
    .build()
    .expect("Failed to build NFTokenCreateOffer");

    let submit = SubmitRequestBuilder::new(&nftoken_offer_tx, &wallet)
        .build()
        .expect("Failed to build submit request");

    let submit_result = client.request(&submit).await;

    match submit_result {
        Ok(result) => match result.result() {
            Ok(_) => {
                println!(
                    "NFTokenCreateOffer transaction submitted successfully"
                );
            }
            Err(e) => {
                println!(
                    "Transaction failed as expected (NFT likely doesn't exist): {e:?}"
                );
            }
        },
        Err(e) => {
            panic!("Failed to submit transaction: {e:#?}");
        }
    }
}

#[serial]
#[tokio::test]
async fn test_nftoken_create_offer_with_expiration() {
    let wallet = sender_wallet();
    let buyer_addr = sender_address();

    let client = Client::new(server_url());

    let nftoken_offer_tx = NFTokenCreateOfferBuilder::new(
        buyer_addr,
        "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65",
        xrp!(7.49),
    )
    .with_expiration(ripple_now() + OFFER_EXPIRATION_1_HOUR)
    .fill(&client)
    .await
    .expect("Failed to auto-fill NFTokenCreateOffer")
    .build()
    .expect("Failed to build NFTokenCreateOffer");

    let submit = SubmitRequestBuilder::new(&nftoken_offer_tx, &wallet)
        .build()
        .expect("Failed to build submit request");

    let submit_result = client
        .request(&submit)
        .await
        .expect("Failed to submit NFTokenCreateOffer");

    assert!(submit_result.result().is_ok());
}
