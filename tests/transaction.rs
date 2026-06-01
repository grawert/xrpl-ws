mod common;

use serial_test::serial;
use ripple_keypairs::Seed;
use xrpl::{Client, xrp};
use xrpl::subscriptions::AccountTransactionsSubscription;
use xrpl::types::builders::{
    PaymentBuilder, SubmitRequestBuilder, TicketCreateBuilder,
};
use common::*;

const PAYMENT_XRP: f64 = 4.49;

#[serial]
#[tokio::test]
async fn test_transaction() {
    let seed = test_seed(1);
    let destination_address = receiver_address();

    let seed: Seed = seed.parse().expect("Seed parse failed");
    let (private_key, public_key) =
        seed.derive_keypair().expect("Key derivation failed");
    let wallet = Wallet { public_key, private_key };

    let client = Client::new(server_url());

    let payment = PaymentBuilder::new(
        wallet.public_key.derive_address(),
        destination_address,
        xrp!(PAYMENT_XRP),
    )
    .fill(&client)
    .await
    .expect("fill failed")
    .build()
    .expect("Payment build failed");

    let result = client
        .request(
            SubmitRequestBuilder::new(&payment, &wallet)
                .build()
                .expect("Signing failed"),
        )
        .await
        .expect("Submit failed")
        .result()
        .expect("Response error");

    assert_accepted(&result, "Payment");
}

#[serial]
#[tokio::test]
async fn test_transaction_with_invoice_id() {
    use sha2::{Digest, Sha256};

    let seed = test_seed(1);
    let destination_address = receiver_address();

    let seed: Seed = seed.parse().expect("Seed parse failed");
    let (private_key, public_key) =
        seed.derive_keypair().expect("Key derivation failed");
    let wallet = Wallet { public_key, private_key };

    let client = Client::new(server_url());

    let invoice_id = hex::encode(Sha256::digest("invoice-2026-001"));

    let payment = PaymentBuilder::new(
        wallet.public_key.derive_address(),
        destination_address,
        xrp!(PAYMENT_XRP),
    )
    .with_invoice_id(&invoice_id)
    .expect("Invalid invoice ID")
    .fill(&client)
    .await
    .expect("fill failed")
    .build()
    .expect("Payment build failed");

    let result = client
        .request(
            SubmitRequestBuilder::new(&payment, &wallet)
                .build()
                .expect("Signing failed"),
        )
        .await
        .expect("Submit failed")
        .result()
        .expect("Response error");

    assert_accepted(&result, "Payment");
}

/// Creates one ticket via `TicketCreate`, waits for it to be validated, then
/// submits a payment that references the ticket instead of a regular sequence number.
#[serial]
#[tokio::test]
async fn test_transaction_with_ticket() {
    let seed = test_seed(1);
    let destination_address = receiver_address();

    let seed: Seed = seed.parse().expect("Seed parse failed");
    let (private_key, public_key) =
        seed.derive_keypair().expect("Key derivation failed");
    let wallet = Wallet { public_key, private_key };
    let account = wallet.public_key.derive_address();

    let client = Client::new(server_url());

    let sub_req =
        AccountTransactionsSubscription::validated(vec![account.clone()])
            .expect("Valid address");
    let (_resp, mut handle) =
        client.subscribe(sub_req).await.expect("Subscription failed");

    // --- Step 1: Create one ticket ---

    let ticket_create_tx = TicketCreateBuilder::new(account.clone(), 1)
        .fill(&client)
        .await
        .expect("fill failed")
        .build()
        .expect("TicketCreate build failed");

    let ticket_create_seq = ticket_create_tx.sequence;
    let ticket_seq = ticket_create_tx.ticket_sequences().unwrap()[0];

    client
        .request(
            SubmitRequestBuilder::new(&ticket_create_tx, &wallet)
                .build()
                .expect("Signing failed"),
        )
        .await
        .expect("Submit failed")
        .result()
        .expect("Response error");

    while let Ok(msg) = handle.recv().await {
        if msg.tx_json.sequence == ticket_create_seq {
            assert_eq!(
                msg.engine_result, "tesSUCCESS",
                "TicketCreate failed: {}",
                msg.engine_result
            );
            break;
        }
    }

    // --- Step 2: Submit a payment using the ticket ---

    let payment = PaymentBuilder::new(
        account.clone(),
        destination_address,
        xrp!(PAYMENT_XRP),
    )
    .with_ticket_sequence(ticket_seq)
    .fill(&client)
    .await
    .expect("fill failed")
    .build()
    .expect("Payment build failed");

    let result = client
        .request(
            SubmitRequestBuilder::new(&payment, &wallet)
                .build()
                .expect("Signing failed"),
        )
        .await
        .expect("Submit failed")
        .result()
        .expect("Response error");

    assert_accepted(&result, "Payment with ticket");

    while let Ok(msg) = handle.recv().await {
        if msg.tx_json.ticket_sequence == Some(ticket_seq) {
            assert_eq!(
                msg.engine_result, "tesSUCCESS",
                "ticket payment not validated: {}",
                msg.engine_result
            );
            break;
        }
    }
}
