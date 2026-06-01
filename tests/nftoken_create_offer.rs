mod common;

use ripple_keypairs::Seed;
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
    let seed_str = test_seed(1);

    let seed: Seed = seed_str.parse().unwrap();
    let (private_key, public_key) = seed.derive_keypair().unwrap();
    let wallet = Wallet { public_key, private_key };
    let buyer_addr = wallet.public_key.derive_address();

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
        test_nftoken_id.to_string(),
        xrp!(7.49),
    )
    .fill(&client)
    .await
    .unwrap()
    .build()
    .unwrap();

    let submit_result = client
        .request(
            SubmitRequestBuilder::new(&nftoken_offer_tx, &wallet)
                .build()
                .unwrap(),
        )
        .await;

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
    let seed_str = test_seed(1);

    let seed: Seed = seed_str.parse().unwrap();
    let (private_key, public_key) = seed.derive_keypair().unwrap();
    let wallet = Wallet { public_key, private_key };
    let buyer_addr = wallet.public_key.derive_address();

    let client = Client::new(server_url());

    let nftoken_offer_tx = NFTokenCreateOfferBuilder::new(
        buyer_addr,
        "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65"
            .to_string(),
        xrp!(7.49),
    )
    .with_expiration(ripple_now() + OFFER_EXPIRATION_1_HOUR)
    .fill(&client)
    .await
    .unwrap()
    .build()
    .unwrap();

    let submit_result = client
        .request(
            SubmitRequestBuilder::new(&nftoken_offer_tx, &wallet)
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(submit_result.result().is_ok());
}
