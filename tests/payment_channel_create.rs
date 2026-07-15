mod common;

use std::time::Duration;
use sha2::{Digest, Sha512};
use serial_test::serial;
use xrpl::types::TransactionType;
use xrpl::subscriptions::AccountTransactionsSubscription;
use xrpl::types::builders::{
    PaymentChannelClaimBuilder, PaymentChannelCreateBuilder,
    SubmitRequestBuilder,
};
use xrpl::types::PaymentChannelClaimAction;
use xrpl::{Client, xrp};
use common::*;

/// Settlement delay after which a sender's `Close` request becomes effective.
const SETTLE_DELAY_1_HOUR: u32 = 3600;
const SETTLE_DELAY_30_MIN: u32 = 1800;

/// Time window (in seconds, ripple epoch) before the channel becomes
/// cancellable. Short enough to let the test teardown close the channel.
const CANCEL_AFTER_SECS: u32 = 14;

/// Sleep to outlast `CANCEL_AFTER_SECS` plus a margin so the close succeeds.
const TEARDOWN_WAIT_SECS: u64 = 17;

/// Computes the PaymentChannelID from the source address, destination address,
/// and the sequence number of the PaymentChannelCreate transaction.
///
/// SHA512Half(uint16_t(0x0078) || src_account_id || dst_account_id || sequence_u32_be)
fn channel_id(src: &str, dst: &str, sequence: u32) -> String {
    let src_id: xrpl_mithril::types::AccountId =
        src.parse().expect("invalid src address");
    let dst_id: xrpl_mithril::types::AccountId =
        dst.parse().expect("invalid dst address");
    let mut data = Vec::with_capacity(46);
    data.extend_from_slice(&0x0078u16.to_be_bytes());
    data.extend_from_slice(src_id.as_ref());
    data.extend_from_slice(dst_id.as_ref());
    data.extend_from_slice(&sequence.to_be_bytes());
    hex::encode(&Sha512::digest(&data)[..32]).to_uppercase()
}

#[serial]
#[tokio::test]
async fn test_payment_channel_create_basic() {
    let destination_address = receiver_address();

    let wallet = sender_wallet();
    let public_key_str = wallet.public_key.to_string();
    let sender_address = sender_address();

    let client = Client::new(server_url());

    let sub_req =
        AccountTransactionsSubscription::validated([sender_address.clone()])
            .expect("Valid address");

    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, mut stream) =
        conn.subscribe(&sub_req).await.expect("Subscription failed");

    let channel_tx = PaymentChannelCreateBuilder::new(
        sender_address.clone(),
        destination_address.clone(),
        public_key_str,
        xrp!(7.49),
        SETTLE_DELAY_1_HOUR,
    )
    .with_cancel_after(ripple_now() + CANCEL_AFTER_SECS)
    .fill(&client)
    .await
    .expect("Failed to auto-fill PaymentChannelCreate")
    .build()
    .expect("Failed to build PaymentChannelCreate");

    let submit = SubmitRequestBuilder::new(&channel_tx, &wallet)
        .build()
        .expect("Failed to build submit request");

    client
        .request(&submit)
        .await
        .expect("Failed to submit PaymentChannelCreate");

    let mut validated_found = false;
    while let Ok(msg) = stream.recv().await {
        if msg.tx_json.sequence == channel_tx.sequence {
            if let TransactionType::PaymentChannelCreate(
                xrpl::types::transactions::payment_channel::PaymentChannelCreate {
                    destination,
                    settle_delay,
                    ..
                },
            ) = &msg.tx_json.transaction_type
            {
                assert_eq!(*destination, destination_address);
                assert_eq!(*settle_delay, SETTLE_DELAY_1_HOUR);
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

    // Teardown: wait for cancel_after to pass, then close the channel to
    // release the owner reserve.
    tokio::time::sleep(Duration::from_secs(TEARDOWN_WAIT_SECS)).await;

    let claim_tx = PaymentChannelClaimBuilder::new(
        sender_address.clone(),
        channel_id(&sender_address, &destination_address, channel_tx.sequence),
    )
    .with_flags(PaymentChannelClaimAction::Close)
    .fill(&client)
    .await
    .expect("Failed to auto-fill PaymentChannelClaim")
    .build()
    .expect("Failed to build PaymentChannelClaim");

    let submit = SubmitRequestBuilder::new(&claim_tx, &wallet)
        .build()
        .expect("Failed to build submit request");

    let result = client
        .request(&submit)
        .await
        .expect("Failed to submit PaymentChannelClaim")
        .result()
        .expect("Failed to get PaymentChannelClaim result");
    assert_accepted(&result, "PaymentChannelClaim close basic");

    while let Ok(msg) = stream.recv().await {
        if msg.tx_json.sequence == claim_tx.sequence {
            assert_eq!(
                msg.engine_result, "tesSUCCESS",
                "PaymentChannelClaim close failed"
            );
            break;
        }
    }
}

#[serial]
#[tokio::test]
async fn test_payment_channel_create_with_cancel_after() {
    let destination_address = receiver_address();

    let wallet = sender_wallet();
    let public_key_str = wallet.public_key.to_string();
    let sender_address = sender_address();

    let client = Client::new(server_url());

    let sub_req =
        AccountTransactionsSubscription::validated([sender_address.clone()])
            .expect("Valid address");

    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, mut stream) =
        conn.subscribe(&sub_req).await.expect("Subscription failed");

    let channel_tx = PaymentChannelCreateBuilder::new(
        sender_address.clone(),
        destination_address.clone(),
        public_key_str,
        xrp!(7.49),
        SETTLE_DELAY_30_MIN,
    )
    .with_destination_tag(98765)
    .with_cancel_after(ripple_now() + CANCEL_AFTER_SECS)
    .fill(&client)
    .await
    .expect("Failed to auto-fill PaymentChannelCreate")
    .build()
    .expect("Failed to build PaymentChannelCreate");

    let submit = SubmitRequestBuilder::new(&channel_tx, &wallet)
        .build()
        .expect("Failed to build submit request");

    client
        .request(&submit)
        .await
        .expect("Failed to submit PaymentChannelCreate");

    while let Ok(msg) = stream.recv().await {
        if msg.tx_json.sequence == channel_tx.sequence {
            assert_eq!(msg.engine_result, "tesSUCCESS");
            break;
        }
    }

    // Teardown: wait for cancel_after to pass, then close the channel to
    // release the owner reserve.
    tokio::time::sleep(Duration::from_secs(TEARDOWN_WAIT_SECS)).await;

    let claim_tx = PaymentChannelClaimBuilder::new(
        sender_address.clone(),
        channel_id(&sender_address, &destination_address, channel_tx.sequence),
    )
    .with_flags(PaymentChannelClaimAction::Close)
    .fill(&client)
    .await
    .expect("Failed to auto-fill PaymentChannelClaim")
    .build()
    .expect("Failed to build PaymentChannelClaim");

    let submit = SubmitRequestBuilder::new(&claim_tx, &wallet)
        .build()
        .expect("Failed to build submit request");

    let result = client
        .request(&submit)
        .await
        .expect("Failed to submit PaymentChannelClaim")
        .result()
        .expect("Failed to get PaymentChannelClaim result");
    assert_accepted(&result, "PaymentChannelClaim close with cancel_after");

    while let Ok(msg) = stream.recv().await {
        if msg.tx_json.sequence == claim_tx.sequence {
            assert_eq!(
                msg.engine_result, "tesSUCCESS",
                "PaymentChannelClaim close failed"
            );
            break;
        }
    }
}
