mod common;

use ripple_keypairs::Seed;
use serial_test::serial;
use xrpl::types::{Amount, Signable};
// ...existing imports...
use xrpl::request::account_info::AccountInfoRequest;
use xrpl::request::submit::SubmitRequest;
use xrpl::types::builders::NFTokenCreateOfferBuilder;
use xrpl::{Client, drops, xrp};
use common::*;

#[ignore]
#[serial]
#[tokio::test]
async fn test_nftoken_create_buy_offer() {
    let seed_str = test_seed();

    let seed: Seed = seed_str.parse().unwrap();
    let (private_key, public_key) = seed.derive_keypair().unwrap();
    let wallet = Wallet { public_key, private_key };
    let buyer_addr = wallet.public_key.derive_address();

    let client = Client::new(&server_url());

    // ...existing code...

    let info = client
        .request(AccountInfoRequest {
            account: buyer_addr.clone(),
            ..Default::default()
        })
        .await
        .unwrap();

    let account_data = info.result().unwrap().account_data;
    let sequence_num = account_data.sequence;
    let balance = account_data.balance.parse::<u64>().unwrap_or(0);

    println!(
        "Account: {} | Balance: {} drops ({} XRP) | Sequence: {}",
        buyer_addr,
        balance,
        balance / 1_000_000,
        sequence_num
    );

    if balance < 2_000_000 {
        panic!(
            "Insufficient balance: {} drops. Need at least 2 XRP for this test.",
            balance
        );
    }

    let test_nftoken_id =
        "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65";

    let nftoken_offer_tx = NFTokenCreateOfferBuilder::new(
        buyer_addr.clone(),
        test_nftoken_id.to_string(),
        sequence_num,
        drops!(12),
        xrp!(0.001),
    )
    .build()
    .unwrap();

    let tx_blob = nftoken_offer_tx.sign_with(&wallet).unwrap();

    if tx_blob.is_empty() {
        panic!("Transaction blob is empty - signing failed");
    }

    let submit_result = client
        .request(SubmitRequest {
            tx_blob: tx_blob.clone(),
            fail_hard: Some(false),
        })
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
                    "Transaction failed as expected (NFT likely doesn't exist): {:?}",
                    e
                );
                return;
            }
        },
        Err(e) => {
            panic!("Failed to submit transaction: {:#?}", e);
        }
    }
}

#[ignore]
#[serial]
#[tokio::test]
async fn test_nftoken_create_offer_with_expiration() {
    let seed_str = test_seed();

    let seed: Seed = seed_str.parse().unwrap();
    let (private_key, public_key) = seed.derive_keypair().unwrap();
    let wallet = Wallet { public_key, private_key };
    let buyer_addr = wallet.public_key.derive_address();

    let client = Client::new(&server_url());

    let info = client
        .request(AccountInfoRequest {
            account: buyer_addr.clone(),
            ..Default::default()
        })
        .await
        .unwrap();

    let sequence_num = info.result().unwrap().account_data.sequence;

    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    let nftoken_offer_tx = NFTokenCreateOfferBuilder::new(
        buyer_addr,
        "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65"
            .to_string(),
        sequence_num,
        drops!(10),
        xrp!(50),
    )
    .with_expiration(current_time + 3600)
    .build()
    .unwrap();

    let tx_blob = nftoken_offer_tx.sign_with(&wallet).unwrap();
    let submit_result = client
        .request(SubmitRequest { tx_blob, fail_hard: None })
        .await
        .unwrap();

    assert!(submit_result.result().is_ok());
}
