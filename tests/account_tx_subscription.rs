mod common;

use ripple_keypairs::Seed;
use serial_test::serial;
use xrpl::types::{Amount, Signable, TransactionType};
use xrpl::subscriptions::AccountTransactionsSubscription;
use xrpl::request::account_info::AccountInfoRequest;
use xrpl::request::submit::SubmitRequest;
use xrpl::types::builders::PaymentBuilder;
use xrpl::{Client, drops, xrp};
use common::*;

#[ignore]
#[serial]
#[tokio::test]
async fn test_transaction_subscription() {
    let seed_str = test_seed();
    let destination_address = receiver_address();

    let seed: Seed = seed_str.parse().unwrap();
    let (private_key, public_key) = seed.derive_keypair().unwrap();
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

    let seq = info.result().unwrap().account_data.sequence;

    let payment = PaymentBuilder::new(
        sender_address,
        destination_address,
        seq,
        drops!(10),
        xrp!(0.01),
    )
    .build()
    .unwrap();

    let blob = payment.sign_with(&wallet).unwrap();
    client
        .request(SubmitRequest { tx_blob: blob, fail_hard: None })
        .await
        .unwrap();

    let mut validated_found = false;
    while let Ok(msg) = handle.recv().await {
        eprintln!(
            "Received tx [{}]: {}",
            if msg.validated { "validated" } else { "unvalidated" },
            msg.hash
        );

        if msg.validated {
            if let TransactionType::Payment { amount, deliver_max, .. } =
                msg.tx_json.transaction_type
            {
                let payment_amount = amount
                    .or(deliver_max)
                    .unwrap_or_else(|| Amount::Xrpl("0".to_string()));

                eprintln!("Transaction amount: {}", payment_amount);
            }

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
