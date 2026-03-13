mod common;

use ripple_keypairs::Seed;
use serial_test::serial;
use xrpl::types::{Amount, Signable, TransactionType};
use xrpl::subscriptions::AccountTransactionsSubscription;
use xrpl::request::account_info::AccountInfoRequest;
use xrpl::request::submit::SubmitRequest;
use xrpl::types::builders::PaymentChannelCreateBuilder;
use xrpl::{Client, drops, xrp};
use common::*;

#[ignore]
#[serial]
#[tokio::test]
async fn test_payment_channel_create_basic() {
    let seed_str = test_seed();
    let destination_address = receiver_address();

    let seed: Seed = seed_str.parse().unwrap();
    let (private_key, public_key) = seed.derive_keypair().unwrap();
    let public_key_str = public_key.to_string();
    let wallet = Wallet { public_key, private_key };
    let sender_address = wallet.public_key.derive_address();

    let client = Client::new(&server_url());

    let sub_req =
        AccountTransactionsSubscription::proposed(vec![sender_address.clone()])
            .expect("Valid address");

    let (_resp, mut handle) =
        client.subscribe(sub_req).await.expect("Subscription failed");

    let info = client
        .request(AccountInfoRequest {
            account: sender_address.clone(),
            ..Default::default()
        })
        .await
        .unwrap();

    let sequence_num = info.result().unwrap().account_data.sequence;

    let channel_tx = PaymentChannelCreateBuilder::new(
        sender_address,
        destination_address.clone(),
        public_key_str,
        sequence_num,
        drops!(10),
        xrp!(100),
        3600,
    )
    .build()
    .unwrap();

    let tx_blob = channel_tx.sign_with(&wallet).unwrap();
    client.request(SubmitRequest { tx_blob, fail_hard: None }).await.unwrap();

    let mut validated_found = false;
    while let Ok(msg) = handle.recv().await {
        if msg.validated {
            if let TransactionType::PaymentChannelCreate {
                destination,
                settle_delay,
                ..
            } = msg.tx_json.transaction_type
            {
                assert_eq!(destination, destination_address);
                assert_eq!(settle_delay, 3600);
            }

            assert_eq!(msg.engine_result, "tesSUCCESS");
            validated_found = true;
            break;
        }
    }

    assert!(
        validated_found,
        "Failed to find validated PaymentChannelCreate transaction"
    );
}

#[ignore]
#[serial]
#[tokio::test]
async fn test_payment_channel_create_with_cancel_after() {
    let seed_str = test_seed();
    let destination_address = receiver_address();

    let seed: Seed = seed_str.parse().unwrap();
    let (private_key, public_key) = seed.derive_keypair().unwrap();
    let public_key_str = public_key.to_string();
    let wallet = Wallet { public_key, private_key };
    let sender_address = wallet.public_key.derive_address();

    let client = Client::new(&server_url());

    let info = client
        .request(AccountInfoRequest {
            account: sender_address.clone(),
            ..Default::default()
        })
        .await
        .unwrap();

    let sequence_num = info.result().unwrap().account_data.sequence;

    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    let channel_tx = PaymentChannelCreateBuilder::new(
        sender_address,
        destination_address,
        public_key_str,
        sequence_num,
        drops!(10),
        xrp!(50),
        1800,
    )
    .with_destination_tag(98765)
    .with_cancel_after(current_time + 86400)
    .build()
    .unwrap();

    let tx_blob = channel_tx.sign_with(&wallet).unwrap();
    let submit_result = client
        .request(SubmitRequest { tx_blob, fail_hard: None })
        .await
        .unwrap();

    assert!(submit_result.result().is_ok());
}
