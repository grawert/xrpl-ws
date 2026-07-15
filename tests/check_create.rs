mod common;

use sha2::{Digest, Sha512};
use serial_test::serial;
use xrpl::types::TransactionType;
use xrpl::subscriptions::AccountTransactionsSubscription;
use xrpl::types::builders::{
    CheckCancelBuilder, CheckCreateBuilder, SubmitRequestBuilder,
};
use xrpl::{Client, xrp};
use common::*;

/// Amount the check is created for, in XRP units.
const CHECK_AMOUNT_XRP: f64 = 7.49;
/// Same amount expressed as drops for round-trip assertions.
const CHECK_AMOUNT_DROPS: &str = "7490000";

/// One-day expiration offset (in seconds) for the cancellable test check.
const EXPIRATION_OFFSET_24H: u32 = 86_400;

/// Computes the CheckID from the sender address and the sequence number of the
/// CheckCreate transaction.
///
/// SHA512Half(uint16_t(0x0043) || account_id || sequence_u32_be)
fn check_id(account: &str, sequence: u32) -> String {
    let account_id: xrpl_mithril::types::AccountId =
        account.parse().expect("invalid address");
    let mut data = Vec::with_capacity(26);
    data.extend_from_slice(&0x0043u16.to_be_bytes());
    data.extend_from_slice(account_id.as_ref());
    data.extend_from_slice(&sequence.to_be_bytes());
    hex::encode(&Sha512::digest(&data)[..32]).to_uppercase()
}

#[serial]
#[tokio::test]
async fn test_check_create_basic() {
    let destination_address = receiver_address();

    let wallet = sender_wallet();
    let sender_addr = sender_address();

    let client = Client::new(server_url());

    let sub_req =
        AccountTransactionsSubscription::validated([sender_addr.clone()])
            .expect("Valid address");

    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, mut stream) =
        conn.subscribe(&sub_req).await.expect("Subscription failed");

    let check_tx = CheckCreateBuilder::new(
        sender_addr.clone(),
        destination_address.clone(),
        xrp!(CHECK_AMOUNT_XRP),
    )
    .fill(&client)
    .await
    .expect("Failed to auto-fill CheckCreate")
    .build()
    .expect("Failed to build CheckCreate");

    let submit = SubmitRequestBuilder::new(&check_tx, &wallet)
        .build()
        .expect("Failed to build submit request");

    client.request(&submit).await.expect("Failed to submit CheckCreate");

    let mut validated_found = false;
    while let Ok(msg) = stream.recv().await {
        if msg.tx_json.sequence == check_tx.sequence {
            if let TransactionType::CheckCreate(
                xrpl::types::transactions::payment::CheckCreate {
                    destination,
                    send_max,
                    ..
                },
            ) = msg.tx_json.transaction_type
            {
                assert_eq!(destination, destination_address);
                assert_eq!(send_max.as_drops(), Some(CHECK_AMOUNT_DROPS));
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

    // Teardown: cancel the check to release the owner reserve.
    let cancel_tx = CheckCancelBuilder::new(
        sender_addr.clone(),
        check_id(&sender_addr, check_tx.sequence),
    )
    .fill(&client)
    .await
    .expect("Failed to auto-fill CheckCancel")
    .build()
    .expect("Failed to build CheckCancel");

    let submit = SubmitRequestBuilder::new(&cancel_tx, &wallet)
        .build()
        .expect("Failed to build submit request");

    let result = client
        .request(&submit)
        .await
        .expect("Failed to submit CheckCancel")
        .result()
        .expect("Failed to get CheckCancel result");

    assert_accepted(&result, "CheckCancel basic");

    while let Ok(msg) = stream.recv().await {
        if msg.tx_json.sequence == cancel_tx.sequence {
            assert_eq!(msg.engine_result, "tesSUCCESS", "CheckCancel failed");
            break;
        }
    }
}

#[serial]
#[tokio::test]
async fn test_check_create_with_expiration() {
    let destination_addr = receiver_address();

    let wallet = sender_wallet();
    let sender_addr = sender_address();

    let client = Client::new(server_url());

    let sub_req =
        AccountTransactionsSubscription::validated([sender_addr.clone()])
            .expect("Valid address");
    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, mut stream) =
        conn.subscribe(&sub_req).await.expect("Subscription failed");

    let mut hasher = Sha512::new();
    hasher.update(b"INV123");
    let invoice_id_hex = hex::encode(&hasher.finalize()[..32]).to_uppercase();

    let check_tx = CheckCreateBuilder::new(
        sender_addr.clone(),
        destination_addr,
        xrp!(CHECK_AMOUNT_XRP),
    )
    .with_destination_tag(54321)
    .with_expiration(ripple_now() + EXPIRATION_OFFSET_24H)
    .with_invoice_id(invoice_id_hex)
    .fill(&client)
    .await
    .expect("Failed to auto-fill CheckCreate")
    .build()
    .expect("Failed to build CheckCreate");

    let submit = SubmitRequestBuilder::new(&check_tx, &wallet)
        .build()
        .expect("Failed to build submit request");

    let submit_result =
        client.request(&submit).await.expect("Failed to submit CheckCreate");

    assert!(submit_result.result().is_ok());

    // Wait for the CheckCreate to be validated.
    while let Ok(msg) = stream.recv().await {
        if msg.tx_json.sequence == check_tx.sequence {
            assert_eq!(msg.engine_result, "tesSUCCESS");
            break;
        }
    }

    // Teardown: cancel the check to release the owner reserve.
    let cancel_tx = CheckCancelBuilder::new(
        sender_addr.clone(),
        check_id(&sender_addr, check_tx.sequence),
    )
    .fill(&client)
    .await
    .expect("Failed to auto-fill CheckCancel")
    .build()
    .expect("Failed to build CheckCancel");

    let submit = SubmitRequestBuilder::new(&cancel_tx, &wallet)
        .build()
        .expect("Failed to build submit request");

    let result = client
        .request(&submit)
        .await
        .expect("Failed to submit CheckCancel")
        .result()
        .expect("Failed to get CheckCancel result");

    assert_accepted(&result, "CheckCancel expiration");

    while let Ok(msg) = stream.recv().await {
        if msg.tx_json.sequence == cancel_tx.sequence {
            assert_eq!(msg.engine_result, "tesSUCCESS", "CheckCancel failed");
            break;
        }
    }
}
