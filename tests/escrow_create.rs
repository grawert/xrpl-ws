mod common;

use std::time::Duration;
use ripple_keypairs::Seed;
use serial_test::serial;
use xrpl::types::TransactionType;
use xrpl::subscriptions::AccountTransactionsSubscription;
use xrpl::types::builders::{
    EscrowCancelBuilder, EscrowCreateBuilder, SubmitRequestBuilder,
};
use xrpl::{Client, xrp};
use common::*;

/// Seconds (from `ripple_now`) until the escrow becomes finishable.
const FINISH_AFTER_SECS: u32 = 10;
/// Seconds (from `ripple_now`) until the escrow becomes cancellable.
const CANCEL_AFTER_SECS: u32 = 14;
/// Sleep before cancelling so `CANCEL_AFTER_SECS` has passed comfortably.
const TEARDOWN_WAIT_SECS: u64 = 17;

#[serial]
#[tokio::test]
async fn test_escrow_create_basic() {
    let seed_str = test_seed(1);
    let destination_address = receiver_address();

    let seed: Seed = seed_str.parse().expect("Failed to parse seed");
    let (private_key, public_key) =
        seed.derive_keypair().expect("Failed to derive keypair");
    let wallet = Wallet { public_key, private_key };
    let sender_addr = wallet.public_key.derive_address();

    let client = Client::new(server_url());

    let sub_req =
        AccountTransactionsSubscription::validated([sender_addr.clone()])
            .expect("Valid address");

    let (_resp, mut handle) =
        client.subscribe(&sub_req).await.expect("Subscription failed");

    let now = ripple_now();

    let escrow_tx = EscrowCreateBuilder::new(
        sender_addr.clone(),
        destination_address.clone(),
        xrp!(7.49),
    )
    .with_finish_after(now + FINISH_AFTER_SECS)
    .with_cancel_after(now + CANCEL_AFTER_SECS)
    .fill(&client)
    .await
    .expect("Failed to auto-fill EscrowCreate")
    .build()
    .expect("Failed to build EscrowCreate");

    client
        .request(
            &SubmitRequestBuilder::new(&escrow_tx, &wallet)
                .build()
                .expect("Failed to build submit request"),
        )
        .await
        .expect("Failed to submit EscrowCreate");

    let mut validated_found = false;
    while let Ok(msg) = handle.recv().await {
        if msg.tx_json.sequence == escrow_tx.sequence {
            if let TransactionType::EscrowCreate(
                xrpl::types::transactions::escrow::EscrowCreate {
                    destination,
                    finish_after,
                    cancel_after,
                    ..
                },
            ) = msg.tx_json.transaction_type
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

    // Teardown: wait for cancel_after to pass, then cancel the escrow to
    // release the owner reserve.
    tokio::time::sleep(Duration::from_secs(TEARDOWN_WAIT_SECS)).await;

    let cancel_tx = EscrowCancelBuilder::new(
        sender_addr.clone(),
        sender_addr.clone(),
        escrow_tx.sequence,
    )
    .fill(&client)
    .await
    .expect("Failed to auto-fill EscrowCancel")
    .build()
    .expect("Failed to build EscrowCancel");

    let result = client
        .request(
            &SubmitRequestBuilder::new(&cancel_tx, &wallet)
                .build()
                .expect("Failed to build submit request"),
        )
        .await
        .expect("Failed to submit EscrowCancel")
        .result()
        .expect("Failed to get EscrowCancel result");
    assert_accepted(&result, "EscrowCancel basic");

    while let Ok(msg) = handle.recv().await {
        if msg.tx_json.sequence == cancel_tx.sequence {
            assert_eq!(msg.engine_result, "tesSUCCESS", "EscrowCancel failed");
            break;
        }
    }
}

#[serial]
#[tokio::test]
async fn test_escrow_create_with_destination_tag() {
    let seed_str = test_seed(1);
    let destination_addr = receiver_address();

    let seed: Seed = seed_str.parse().expect("Failed to parse seed");
    let (private_key, public_key) =
        seed.derive_keypair().expect("Failed to derive keypair");
    let wallet = Wallet { public_key, private_key };
    let sender_addr = wallet.public_key.derive_address();

    let client = Client::new(server_url());

    let sub_req =
        AccountTransactionsSubscription::validated([sender_addr.clone()])
            .expect("Valid address");
    let (_resp, mut handle) =
        client.subscribe(&sub_req).await.expect("Subscription failed");

    let now = ripple_now();

    let escrow_tx = EscrowCreateBuilder::new(
        sender_addr.clone(),
        destination_addr,
        xrp!(7.49),
    )
    .with_destination_tag(12345)
    .with_finish_after(now + FINISH_AFTER_SECS)
    .with_cancel_after(now + CANCEL_AFTER_SECS)
    .fill(&client)
    .await
    .expect("Failed to auto-fill EscrowCreate")
    .build()
    .expect("Failed to build EscrowCreate");

    let submit_result = client
        .request(
            &SubmitRequestBuilder::new(&escrow_tx, &wallet)
                .build()
                .expect("Failed to build submit request"),
        )
        .await
        .expect("Failed to submit EscrowCreate");

    assert!(submit_result.result().is_ok());

    while let Ok(msg) = handle.recv().await {
        if msg.tx_json.sequence == escrow_tx.sequence {
            assert_eq!(msg.engine_result, "tesSUCCESS");
            break;
        }
    }

    // Teardown: wait for cancel_after to pass, then cancel the escrow to
    // release the owner reserve.
    tokio::time::sleep(Duration::from_secs(TEARDOWN_WAIT_SECS)).await;

    let cancel_tx = EscrowCancelBuilder::new(
        sender_addr.clone(),
        sender_addr.clone(),
        escrow_tx.sequence,
    )
    .fill(&client)
    .await
    .expect("Failed to auto-fill EscrowCancel")
    .build()
    .expect("Failed to build EscrowCancel");

    let result = client
        .request(
            &SubmitRequestBuilder::new(&cancel_tx, &wallet)
                .build()
                .expect("Failed to build submit request"),
        )
        .await
        .expect("Failed to submit EscrowCancel")
        .result()
        .expect("Failed to get EscrowCancel result");
    assert_accepted(&result, "EscrowCancel destination tag");

    while let Ok(msg) = handle.recv().await {
        if msg.tx_json.sequence == cancel_tx.sequence {
            assert_eq!(msg.engine_result, "tesSUCCESS", "EscrowCancel failed");
            break;
        }
    }
}
