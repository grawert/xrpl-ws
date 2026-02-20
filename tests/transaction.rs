mod common;

use anyhow::{anyhow, Result};
use hex;
use ripple_keypairs::{PrivateKey, PublicKey, Seed};
use rippled_binary_codec::serialize::serialize_tx;
use xrpl::*;
use xrpl::request::account_info::AccountInfoRequest;
use xrpl::request::submit::SubmitRequest;
use xrpl::types::*;
use common::*;

const STX_PREFIX: &str = "53545800";
const FEE_DROPS: u32 = 10;
const PAYMENT_XRP: f64 = 0.01;

pub struct Wallet {
    pub public_key: PublicKey,
    pub private_key: PrivateKey,
}

impl SigningContext for Wallet {
    type Error = anyhow::Error;

    fn sign_transaction(
        &self,
        tx: &Transaction,
    ) -> Result<String, Self::Error> {
        let mut tx_json = serde_json::to_value(tx)
            .expect("Failed to convert transaction to json");
        tx_json["SigningPubKey"] = self.public_key.to_string().into();

        let tx_hex = serialize_tx(serde_json::to_string(&tx_json)?, true)
            .ok_or_else(|| {
                anyhow!("Failed to serialize transaction for signing")
            })?;

        let signing_bytes = hex::decode(format!("{}{}", STX_PREFIX, tx_hex))?;
        let signature = self.private_key.sign(&signing_bytes);
        tx_json["TxnSignature"] = signature.to_string().into();

        let tx_signed = serialize_tx(serde_json::to_string(&tx_json)?, false)
            .ok_or_else(|| {
            anyhow!("Failed to serialize signed transaction")
        })?;

        Ok(tx_signed)
    }
}

#[ignore]
#[tokio::test]
async fn test_transaction() {
    let seed = std::env::var("TEST_SEED")
        .expect("requires TEST_SEED environment variable");
    let destination_address = std::env::var("TEST_DST_ADDRESS")
        .expect("requires TEST_DST_ADDRESS environment variable");

    let seed: Seed = seed.parse().expect("Seed parse failed");
    let (private_key, public_key) =
        seed.derive_keypair().expect("Key derivation failed");
    let wallet = Wallet { public_key, private_key };

    let client =
        XrplClient::new(SERVER_URL).await.expect("Client creation failed");

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
