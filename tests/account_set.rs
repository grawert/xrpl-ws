mod common;

use ripple_keypairs::Seed;
use serial_test::serial;
use xrpl::types::{Amount, Signable, TransactionType};
use xrpl::subscriptions::AccountTransactionsSubscription;
use xrpl::request::account_info::AccountInfoRequest;
use xrpl::request::submit::SubmitRequest;
use xrpl::types::builders::AccountSetBuilder;
use xrpl::{Client, drops};
use common::*;

#[ignore]
#[serial]
#[tokio::test]
async fn test_account_set_domain() {
    let seed_str = test_seed();

    let seed: Seed = seed_str.parse().unwrap();
    let (private_key, public_key) = seed.derive_keypair().unwrap();
    let wallet = Wallet { public_key, private_key };
    let account_address = wallet.public_key.derive_address();

    let client = Client::new(&server_url());

    let sub_req = AccountTransactionsSubscription::proposed(vec![
        account_address.clone(),
    ])
    .expect("Valid address");

    let (_resp, mut handle) =
        client.subscribe(sub_req).await.expect("Subscription failed");

    let info = client
        .request(AccountInfoRequest {
            account: account_address.clone(),
            ..Default::default()
        })
        .await
        .unwrap();

    let sequence_num = info.result().unwrap().account_data.sequence;

    let domain_hex = hex::encode("example.com").to_uppercase();

    let account_set_tx = AccountSetBuilder::new(
        account_address.clone(),
        sequence_num,
        drops!(10),
    )
    .with_domain(domain_hex.clone())
    .build()
    .unwrap();

    let tx_blob = account_set_tx.sign_with(&wallet).unwrap();
    client.request(SubmitRequest { tx_blob, fail_hard: None }).await.unwrap();

    let mut validated_found = false;
    while let Ok(msg) = handle.recv().await {
        if msg.validated {
            if let TransactionType::AccountSet { domain, .. } =
                msg.tx_json.transaction_type
            {
                assert_eq!(domain, Some(domain_hex));
            }

            assert_eq!(msg.engine_result, "tesSUCCESS");
            validated_found = true;
            break;
        }
    }

    assert!(validated_found, "Failed to find validated AccountSet transaction");
}

#[ignore]
#[serial]
#[tokio::test]
async fn test_account_set_clear_flag() {
    let seed_str = test_seed();

    let seed: Seed = seed_str.parse().unwrap();
    let (private_key, public_key) = seed.derive_keypair().unwrap();
    let wallet = Wallet { public_key, private_key };
    let account_address = wallet.public_key.derive_address();

    let client = Client::new(&server_url());

    let info = client
        .request(AccountInfoRequest {
            account: account_address.clone(),
            ..Default::default()
        })
        .await
        .unwrap();

    let sequence_num = info.result().unwrap().account_data.sequence;

    let account_set_tx =
        AccountSetBuilder::new(account_address, sequence_num, drops!(10))
            .with_clear_flag(1)
            .with_transfer_rate(1000000000)
            .build()
            .unwrap();

    let tx_blob = account_set_tx.sign_with(&wallet).unwrap();
    let submit_result = client
        .request(SubmitRequest { tx_blob, fail_hard: None })
        .await
        .unwrap();

    assert!(submit_result.result().is_ok());
}
