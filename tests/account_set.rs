mod common;

use serial_test::serial;
use xrpl::types::TransactionType;
use xrpl::subscriptions::AccountTransactionsSubscription;
use xrpl::types::builders::{AccountSetBuilder, SubmitRequestBuilder};
use xrpl::types::AccountFlag;
use xrpl::Client;
use common::*;

/// TransferRate is denominated in billionths; 1_000_000_000 is the 1.0x unit
/// value, i.e. no fee charged on issued-currency transfers.
const TRANSFER_RATE_NONE: u32 = 1_000_000_000;

#[serial]
#[tokio::test]
async fn test_account_set_clear_flag() {
    let wallet = sender_wallet();
    let account_address = sender_address();

    let client = Client::new(server_url());

    let sub_req =
        AccountTransactionsSubscription::validated([account_address.clone()])
            .expect("Valid address");

    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, mut stream) =
        conn.subscribe(&sub_req).await.expect("Subscription failed");

    let account_set_tx = AccountSetBuilder::new(account_address)
        .with_clear_flag(AccountFlag::RequireDest)
        .with_transfer_rate(TRANSFER_RATE_NONE)
        .fill(&client)
        .await
        .expect("Failed to auto-fill AccountSet")
        .build()
        .expect("Failed to build AccountSet");

    let submit = SubmitRequestBuilder::new(&account_set_tx, &wallet)
        .build()
        .expect("Failed to build submit request");

    let submit_result =
        client.request(&submit).await.expect("Failed to submit AccountSet");

    assert!(submit_result.result().is_ok());

    while let Ok(msg) = stream.recv().await {
        if msg.tx_json.sequence == account_set_tx.sequence {
            assert_eq!(msg.engine_result, "tesSUCCESS");
            break;
        }
    }
}

#[serial]
#[tokio::test]
async fn test_account_set_domain() {
    let wallet = sender_wallet();
    let account_address = sender_address();

    let client = Client::new(server_url());

    let sub_req =
        AccountTransactionsSubscription::validated([account_address.clone()])
            .expect("Valid address");

    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, mut stream) =
        conn.subscribe(&sub_req).await.expect("Subscription failed");

    let domain_hex = hex::encode("crates.io/crates/xrpl-ws").to_uppercase();

    let account_set_tx = AccountSetBuilder::new(account_address.clone())
        .with_domain(domain_hex.clone())
        .fill(&client)
        .await
        .expect("Failed to auto-fill AccountSet")
        .build()
        .expect("Failed to build AccountSet");

    let submit = SubmitRequestBuilder::new(&account_set_tx, &wallet)
        .build()
        .expect("Failed to build submit request");

    client.request(&submit).await.expect("Failed to submit AccountSet");

    let mut validated_found = false;
    while let Ok(msg) = stream.recv().await {
        if msg.tx_json.sequence == account_set_tx.sequence {
            if let TransactionType::AccountSet(
                xrpl::types::transactions::account::AccountSet {
                    domain, ..
                },
            ) = &msg.tx_json.transaction_type
            {
                assert_eq!(domain.as_deref(), Some(domain_hex.as_str()));
            }
            assert_eq!(msg.engine_result, "tesSUCCESS");
            validated_found = true;
            break;
        }
    }

    assert!(validated_found, "Failed to find validated AccountSet transaction");
}
