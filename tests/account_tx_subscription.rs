mod common;

use ripple_keypairs::Seed;
use serial_test::serial;
use xrpl::types::{Amount, Signable, SigningContext};
use xrpl::subscriptions::AccountTransactionsSubscription;
use xrpl::request::account_info::AccountInfoRequest;
use xrpl::request::submit::SubmitRequest;
use xrpl::types::builders::PaymentBuilder;
use xrpl::{XrplClient, drops, xrp};
use common::*;

pub struct Wallet {
    pub public_key: ripple_keypairs::PublicKey,
    pub private_key: ripple_keypairs::PrivateKey,
}

impl SigningContext for Wallet {
    type Error = anyhow::Error;

    fn sign_transaction(
        &self,
        tx: &xrpl::types::Transaction,
    ) -> anyhow::Result<String, Self::Error> {
        let mut tx_json =
            serde_json::to_value(tx).expect("Failed to convert to json");
        tx_json["SigningPubKey"] = self.public_key.to_string().into();

        let tx_hex = rippled_binary_codec::serialize::serialize_tx(
            serde_json::to_string(&tx_json)?,
            true,
        )
        .ok_or_else(|| anyhow::anyhow!("Failed to serialize tx"))?;

        let signing_bytes = hex::decode(format!("53545800{}", tx_hex))?;
        let signature = self.private_key.sign(&signing_bytes);
        tx_json["TxnSignature"] = signature.to_string().into();

        let tx_signed = rippled_binary_codec::serialize::serialize_tx(
            serde_json::to_string(&tx_json)?,
            false,
        )
        .ok_or_else(|| anyhow::anyhow!("Failed to serialize signed tx"))?;

        Ok(tx_signed)
    }
}

#[ignore]
#[serial]
#[tokio::test]
async fn test_transaction_subscription() {
    let seed_str = std::env::var("TEST_SEED").expect("requires TEST_SEED");
    let dest =
        std::env::var("TEST_DST_ADDRESS").expect("requires TEST_DST_ADDRESS");

    let seed: Seed = seed_str.parse().unwrap();
    let (private_key, public_key) = seed.derive_keypair().unwrap();
    let wallet = Wallet { public_key, private_key };
    let account_addr = wallet.public_key.derive_address();

    let client = XrplClient::new(SERVER_URL).await.expect("Client failed");

    let sub_req =
        AccountTransactionsSubscription::new(vec![account_addr.clone()])
            .expect("Valid address");

    let (_resp, mut receiver) =
        client.subscribe(sub_req).await.expect("Subscription failed");

    let info = client
        .request(AccountInfoRequest {
            account: account_addr.clone(),
            ..Default::default()
        })
        .await
        .unwrap();

    let seq = info.result().unwrap().account_data.sequence;

    let payment =
        PaymentBuilder::new(account_addr, dest, seq, drops!(10), xrp!(0.01))
            .build()
            .unwrap();

    let blob = payment.sign_with(&wallet).unwrap();
    client
        .request(SubmitRequest { tx_blob: blob, fail_hard: None })
        .await
        .unwrap();

    let mut validated_found = false;
    while let Ok(msg) = receiver.receiver().recv().await {
        if msg.validated {
            assert_eq!(msg.engine_result, "tesSUCCESS");
            validated_found = true;
            break;
        }
    }

    assert!(
        validated_found,
        "Timed out or failed to find a validated transaction"
    );
}
