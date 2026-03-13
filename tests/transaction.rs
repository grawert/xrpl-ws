mod common;

use hex;
use serial_test::serial;
use ripple_keypairs::Seed;
use xrpl::*;
use xrpl::request::account_info::AccountInfoRequest;
use xrpl::request::submit::SubmitRequest;
use xrpl::types::*;
use common::*;

const FEE_DROPS: u32 = 10;
const PAYMENT_XRP: f64 = 0.01;

#[ignore]
#[serial]
#[tokio::test]
async fn test_transaction() {
    let seed = test_seed();
    let destination_address = receiver_address();

    let seed: Seed = seed.parse().expect("Seed parse failed");
    let (private_key, public_key) =
        seed.derive_keypair().expect("Key derivation failed");
    let wallet = Wallet { public_key, private_key };

    let client = Client::new(&server_url());

    let response = client
        .request(AccountInfoRequest {
            account: wallet.public_key.derive_address().into(),
            ..Default::default()
        })
        .await
        .expect("account_info failed");

    let sequence =
        response.result().expect("Expected result").account_data.sequence;

    let payment = PaymentBuilder::new(
        wallet.public_key.derive_address().into(),
        destination_address,
        sequence,
        drops!(FEE_DROPS),
        xrp!(PAYMENT_XRP),
    )
    .build()
    .expect("Payment build failed");

    let signed_blob = payment.sign_with(&wallet).expect("Signing failed");

    let response = client
        .request(SubmitRequest { tx_blob: signed_blob, fail_hard: None })
        .await
        .expect("Submit failed");

    let result = response.result().expect("Response error");
    assert_eq!(result.engine_result, "tesSUCCESS");
}

#[ignore]
#[serial]
#[tokio::test]
async fn test_transaction_with_invoice_id() {
    use sha2::{Sha256, Digest};

    let seed = test_seed();
    let destination_address = receiver_address();

    let seed: Seed = seed.parse().expect("Seed parse failed");
    let (private_key, public_key) =
        seed.derive_keypair().expect("Key derivation failed");
    let wallet = Wallet { public_key, private_key };

    let client = Client::new(&server_url());

    let response = client
        .request(AccountInfoRequest {
            account: wallet.public_key.derive_address().into(),
            ..Default::default()
        })
        .await
        .expect("account_info failed");

    let sequence =
        response.result().expect("Expected result").account_data.sequence;

    let invoice_id = hex::encode(Sha256::digest("invoice-2026-001"));

    let payment = PaymentBuilder::new(
        wallet.public_key.derive_address().into(),
        destination_address,
        sequence,
        drops!(FEE_DROPS),
        xrp!(PAYMENT_XRP),
    )
    .with_invoice_id(&invoice_id)
    .build()
    .expect("Payment build failed");

    let signed_blob = payment.sign_with(&wallet).expect("Signing failed");

    let response = client
        .request(SubmitRequest { tx_blob: signed_blob, fail_hard: None })
        .await
        .expect("Submit failed");

    let result = response.result().expect("Response error");
    assert_eq!(result.engine_result, "tesSUCCESS");
}
