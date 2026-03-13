mod common;

use sha2::{Digest, Sha512};
use ripple_keypairs::Seed;
use serial_test::serial;
use xrpl::types::{Amount, Signable, TransactionType};
use xrpl::subscriptions::AccountTransactionsSubscription;
use xrpl::request::account_info::AccountInfoRequest;
use xrpl::request::submit::SubmitRequest;
use xrpl::types::builders::CheckCreateBuilder;
use xrpl::{Client, drops, xrp};
use common::*;

#[ignore]
#[serial]
#[tokio::test]
async fn test_check_create_basic() {
    let seed_str = test_seed();
    let destination_address = receiver_address();

    let seed: Seed = seed_str.parse().unwrap();
    let (private_key, public_key) = seed.derive_keypair().unwrap();
    let wallet = Wallet { public_key, private_key };
    let sender_addr = wallet.public_key.derive_address();

    let client = Client::new(&server_url());

    let sub_req =
        AccountTransactionsSubscription::proposed(vec![sender_addr.clone()])
            .expect("Valid address");

    let (_resp, mut handle) =
        client.subscribe(sub_req).await.expect("Subscription failed");

    let info = client
        .request(AccountInfoRequest {
            account: sender_addr.clone(),
            ..Default::default()
        })
        .await
        .unwrap();

    let sequence_num = info.result().unwrap().account_data.sequence;

    let check_tx = CheckCreateBuilder::new(
        sender_addr,
        destination_address.clone(),
        sequence_num,
        drops!(10),
        xrp!(25),
    )
    .build()
    .unwrap();

    let tx_blob = check_tx.sign_with(&wallet).unwrap();
    client.request(SubmitRequest { tx_blob, fail_hard: None }).await.unwrap();

    let mut validated_found = false;
    while let Ok(msg) = handle.recv().await {
        if msg.validated {
            if let TransactionType::CheckCreate {
                destination, send_max, ..
            } = msg.tx_json.transaction_type
            {
                assert_eq!(destination, destination_address);
                assert_eq!(send_max.as_drops(), Some("25000000"));
            }

            assert_eq!(msg.engine_result, "tesSUCCESS");
            validated_found = true;
            break;
        }
    }

    assert!(
        validated_found,
        "Failed to find validated CheckCreate transaction"
    );
}

#[ignore]
#[serial]
#[tokio::test]
async fn test_check_create_with_expiration() {
    let seed_str = test_seed();
    let destination_addr = receiver_address();

    let seed: Seed = seed_str.parse().unwrap();
    let (private_key, public_key) = seed.derive_keypair().unwrap();
    let wallet = Wallet { public_key, private_key };
    let sender_addr = wallet.public_key.derive_address();

    let client = Client::new(&server_url());

    let info = client
        .request(AccountInfoRequest {
            account: sender_addr.clone(),
            ..Default::default()
        })
        .await
        .unwrap();

    let sequence_num = info.result().unwrap().account_data.sequence;

    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    let invoice_id_input = "INV123";
    let mut hasher = Sha512::new();
    hasher.update(invoice_id_input.as_bytes());
    let hash = hasher.finalize();
    let invoice_id_hash = &hash[..32];
    let invoice_id_hex = hex::encode(invoice_id_hash).to_uppercase();

    let check_tx = CheckCreateBuilder::new(
        sender_addr,
        destination_addr,
        sequence_num,
        drops!(10),
        xrp!(10),
    )
    .with_destination_tag(54321)
    .with_expiration(current_time + 86400)
    .with_invoice_id(invoice_id_hex)
    .build()
    .unwrap();

    let tx_blob = match check_tx.sign_with(&wallet) {
        Ok(blob) => blob,
        Err(e) => {
            eprintln!("Failed to sign transaction: {}", e);
            panic!("Transaction signing failed: {}", e);
        }
    };
    let submit_result = client
        .request(SubmitRequest { tx_blob, fail_hard: None })
        .await
        .unwrap();

    assert!(submit_result.result().is_ok());
}
