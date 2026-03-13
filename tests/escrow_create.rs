mod common;

use ripple_keypairs::Seed;
use serial_test::serial;
use xrpl::types::{Amount, Signable, TransactionType};
use xrpl::subscriptions::AccountTransactionsSubscription;
use xrpl::request::account_info::AccountInfoRequest;
use xrpl::request::submit::SubmitRequest;
use xrpl::types::builders::EscrowCreateBuilder;
use xrpl::{Client, drops, xrp};
use common::*;

#[ignore]
#[serial]
#[tokio::test]
async fn test_escrow_create_basic() {
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

    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    let escrow_tx = EscrowCreateBuilder::new(
        sender_addr,
        destination_address.clone(),
        sequence_num,
        drops!(10),
        xrp!(5),
    )
    .with_finish_after(current_time + 3600)
    .with_cancel_after(current_time + 7200)
    .build()
    .unwrap();

    let tx_blob = escrow_tx.sign_with(&wallet).unwrap();
    client.request(SubmitRequest { tx_blob, fail_hard: None }).await.unwrap();

    let mut validated_found = false;
    while let Ok(msg) = handle.recv().await {
        if msg.validated {
            if let TransactionType::EscrowCreate {
                destination,
                finish_after,
                cancel_after,
                ..
            } = msg.tx_json.transaction_type
            {
                assert_eq!(destination, destination_address);
                assert!(finish_after.is_some());
                assert!(cancel_after.is_some());
            }

            assert_eq!(msg.engine_result, "tesSUCCESS");
            validated_found = true;
            break;
        }
    }

    assert!(
        validated_found,
        "Failed to find validated EscrowCreate transaction"
    );
}

#[ignore]
#[serial]
#[tokio::test]
async fn test_escrow_create_with_destination_tag() {
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

    let escrow_tx = EscrowCreateBuilder::new(
        sender_addr,
        destination_addr,
        sequence_num,
        drops!(10),
        xrp!(1),
    )
    .with_destination_tag(12345)
    .with_finish_after(current_time + 3600)
    .build()
    .unwrap();

    let tx_blob = escrow_tx.sign_with(&wallet).unwrap();
    let submit_result = client
        .request(SubmitRequest { tx_blob, fail_hard: None })
        .await
        .unwrap();

    assert!(submit_result.result().is_ok());
}
