mod common;

use ripple_keypairs::Seed;
use serial_test::serial;
use xrpl::types::HasTransactionMeta;
use xrpl::subscriptions::AccountTransactionsSubscription;
use xrpl::types::builders::{PaymentBuilder, SubmitRequestBuilder};
use xrpl::{Client, xrp};
use common::*;

#[serial]
#[tokio::test]
async fn test_transaction_subscription() {
    let seed_str = test_seed(1);
    let destination_address = receiver_address();

    let seed: Seed = seed_str.parse().unwrap();
    let (private_key, public_key) = seed.derive_keypair().unwrap();
    let wallet = Wallet { public_key, private_key };
    let sender_address = wallet.public_key.derive_address();

    let client = Client::new(server_url());

    let sub_req =
        AccountTransactionsSubscription::proposed([sender_address.clone()])
            .expect("Valid address");

    let (_resp, mut handle) =
        client.subscribe(&sub_req).await.expect("Subscription failed");

    let payment = PaymentBuilder::new(
        sender_address.clone(),
        destination_address.clone(),
        xrp!(7.49),
    )
    .fill(&client)
    .await
    .expect("Failed to auto-fill payment")
    .build()
    .expect("Failed to build payment");

    client
        .request(
            &SubmitRequestBuilder::new(&payment, &wallet)
                .build()
                .expect("Failed to build submit request"),
        )
        .await
        .expect("Failed to submit payment");

    let mut validated_found = false;
    while let Ok(msg) = handle.recv().await {
        eprintln!(
            "Received tx [{}]: {}",
            if msg.validated { "validated" } else { "unvalidated" },
            msg.hash
        );

        if msg.validated {
            if let Some(amount) = msg.delivered_amount() {
                eprintln!(
                    "Transaction from {sender_address} to {destination_address}: {amount}"
                );
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
