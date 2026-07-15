mod common;

use ripple_keypairs::Seed;
use serial_test::serial;
use xrpl::subscriptions::AccountTransactionsSubscription;
use xrpl::types::builders::{
    PaymentBuilder, SignerListSetBuilder, SubmitMultisignedRequestBuilder,
    SubmitRequestBuilder, TicketCreateBuilder,
};
use xrpl::types::{Amount, MultiSignable, SignerEntry, SignerEntryWrapper};
use xrpl::{Client, xrp};
use common::*;

const PAYMENT_AMOUNT_XRP: f64 = 49.99;

fn wallet_for_seed(n: usize) -> Wallet {
    let seed: Seed = test_seed(n)
        .parse()
        .unwrap_or_else(|e| panic!("Failed to parse seed {n}: {e:?}"));
    let (private_key, public_key) = seed
        .derive_keypair()
        .unwrap_or_else(|e| panic!("Failed to derive keypair {n}: {e:?}"));
    Wallet { public_key, private_key }
}

/// Sets up a 2-of-2 signer list on account 2 (TEST_SEED_2), submits a multi-signed
/// payment to account 1 co-signed by accounts 1 and 3, then removes the signer list as teardown.
#[serial]
#[tokio::test]
async fn test_multi_sign_payment() {
    let wallet1 = wallet_for_seed(1);
    let account1 = wallet1.public_key.derive_address();

    let wallet2 = wallet_for_seed(2);
    let account2 = wallet2.public_key.derive_address();

    let wallet3 = wallet_for_seed(3);
    let account3 = wallet3.public_key.derive_address();

    let client = Client::new(server_url());

    let tx = AccountTransactionsSubscription::validated(vec![account2.clone()])
        .expect("Valid address");
    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, mut stream) =
        conn.subscribe(&tx).await.expect("Subscription failed");

    // --- Step 1: Set up a 2-of-2 signer list on account 2 ---

    let signer_list = SignerListSetBuilder::new(account2.clone(), 2)
        .add_signer_entry(SignerEntryWrapper {
            signer_entry: SignerEntry {
                account: account1.clone(),
                signer_weight: 1,
                wallet_locator: None,
            },
        })
        .add_signer_entry(SignerEntryWrapper {
            signer_entry: SignerEntry {
                account: account3.clone(),
                signer_weight: 1,
                wallet_locator: None,
            },
        })
        .fill(&client)
        .await
        .expect("Failed to auto-fill SignerListSet")
        .build()
        .expect("Failed to build SignerListSet");

    let submit = SubmitRequestBuilder::new(&signer_list, &wallet2)
        .build()
        .expect("Failed to build submit request");

    client.request(&submit).await.expect("Failed to submit SignerListSet");

    while let Ok(msg) = stream.recv().await {
        if msg.tx_json.sequence == signer_list.sequence {
            assert_eq!(
                msg.engine_result, "tesSUCCESS",
                "SignerListSet failed: {}",
                msg.engine_result
            );
            break;
        }
    }

    // --- Step 2: Build and submit a multi-signed payment from account 2 to account 1 ---
    //
    // Multi-signing fee: (1 + number_of_signers) x base_fee.
    // We fill() to get the current open_ledger_fee, then multiply by (1 + 2 signers).

    let payment_builder = PaymentBuilder::new(
        account2.clone(),
        account1.clone(),
        xrp!(PAYMENT_AMOUNT_XRP),
    )
    .fill(&client)
    .await
    .expect("Failed to auto-fill payment");

    let base_fee: u64 = match payment_builder.fee() {
        Amount::Xrpl(s) => s.parse().expect("Fee is not a valid u64"),
        _ => unreachable!("fee is always XRP drops"),
    };
    let multisign_fee = Amount::Xrpl(((1 + 2) * base_fee).to_string());

    let mut payment = payment_builder
        .with_fee(multisign_fee)
        .build()
        .expect("Failed to build payment");

    let sig1 = payment.sign_as(&wallet1).expect("Failed to sign as wallet 1");
    let sig3 = payment.sign_as(&wallet3).expect("Failed to sign as wallet 3");
    payment.add_signature(sig1);
    payment.add_signature(sig3);

    let submit_request = SubmitMultisignedRequestBuilder::new(&payment)
        .build()
        .expect("Failed to serialize multi-signed payment");
    let ms_result = client
        .request(&submit_request)
        .await
        .expect("Failed to submit multi-signed payment")
        .result()
        .expect("Failed to get multi-signed payment result");

    let code = &ms_result.engine_result;
    assert!(
        !code.starts_with("tem"),
        "multi-signed payment malformed ({code})"
    );
    assert!(
        code.starts_with("tes"),
        "multi-signed payment failed: engine_result = {code}"
    );

    while let Ok(msg) = stream.recv().await {
        if msg.tx_json.sequence == payment.sequence {
            assert_eq!(
                msg.engine_result, "tesSUCCESS",
                "multi-signed payment not validated: {}",
                msg.engine_result
            );
            break;
        }
    }

    // --- Teardown: remove the signer list so account 2 reverts to single-key signing ---

    let remove = SignerListSetBuilder::new(account2.clone(), 0)
        .fill(&client)
        .await
        .expect("Failed to auto-fill SignerListSet removal")
        .build()
        .expect("Failed to build SignerListSet removal");

    let submit = SubmitRequestBuilder::new(&remove, &wallet2)
        .build()
        .expect("Failed to build submit request");

    client
        .request(&submit)
        .await
        .expect("Failed to submit SignerListSet removal");

    while let Ok(msg) = stream.recv().await {
        if msg.tx_json.sequence == remove.sequence {
            assert_eq!(
                msg.engine_result, "tesSUCCESS",
                "SignerList removal failed: {}",
                msg.engine_result
            );
            break;
        }
    }
}

/// Sets up a 2-of-2 signer list on account 2, creates a ticket on account 2, then
/// submits a multi-signed payment that consumes the ticket, co-signed by accounts 1 and 3.
#[serial]
#[tokio::test]
async fn test_multi_sign_payment_with_ticket() {
    let wallet1 = wallet_for_seed(1);
    let account1 = wallet1.public_key.derive_address();

    let wallet2 = wallet_for_seed(2);
    let account2 = wallet2.public_key.derive_address();

    let wallet3 = wallet_for_seed(3);
    let account3 = wallet3.public_key.derive_address();

    let client = Client::new(server_url());

    let tx = AccountTransactionsSubscription::validated(vec![account2.clone()])
        .expect("Valid address");
    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");
    let (_resp, mut stream) =
        conn.subscribe(&tx).await.expect("Subscription failed");

    // --- Step 1: Set up a 2-of-2 signer list on account 2 ---

    let signer_list = SignerListSetBuilder::new(account2.clone(), 2)
        .add_signer_entry(SignerEntryWrapper {
            signer_entry: SignerEntry {
                account: account1.clone(),
                signer_weight: 1,
                wallet_locator: None,
            },
        })
        .add_signer_entry(SignerEntryWrapper {
            signer_entry: SignerEntry {
                account: account3.clone(),
                signer_weight: 1,
                wallet_locator: None,
            },
        })
        .fill(&client)
        .await
        .expect("Failed to auto-fill SignerListSet")
        .build()
        .expect("Failed to build SignerListSet");

    let submit = SubmitRequestBuilder::new(&signer_list, &wallet2)
        .build()
        .expect("Failed to build submit request");

    client.request(&submit).await.expect("Failed to submit SignerListSet");

    while let Ok(msg) = stream.recv().await {
        if msg.tx_json.sequence == signer_list.sequence {
            assert_eq!(
                msg.engine_result, "tesSUCCESS",
                "SignerListSet failed: {}",
                msg.engine_result
            );
            break;
        }
    }

    // --- Step 2: Create a ticket on account 2 ---

    let ticket_create_tx = TicketCreateBuilder::new(account2.clone(), 1)
        .fill(&client)
        .await
        .expect("Failed to auto-fill TicketCreate")
        .build()
        .expect("Failed to build TicketCreate");

    let ticket_create_seq = ticket_create_tx.sequence;
    let ticket_seq = ticket_create_tx
        .ticket_sequences()
        .expect("Failed to get ticket sequences")[0];

    let submit = SubmitRequestBuilder::new(&ticket_create_tx, &wallet2)
        .build()
        .expect("Failed to build submit request");

    client.request(&submit).await.expect("Failed to submit TicketCreate");

    while let Ok(msg) = stream.recv().await {
        if msg.tx_json.sequence == ticket_create_seq {
            assert_eq!(
                msg.engine_result, "tesSUCCESS",
                "TicketCreate failed: {}",
                msg.engine_result
            );
            break;
        }
    }

    // --- Step 3: Build and submit a multi-signed payment using the ticket ---
    //
    // with_ticket_sequence before fill() skips the account_info round-trip and
    // sets Sequence = 0 automatically. The multi-sign fee is still (1 + N) x base_fee.

    let payment_builder = PaymentBuilder::new(
        account2.clone(),
        account1.clone(),
        xrp!(PAYMENT_AMOUNT_XRP),
    )
    .with_ticket_sequence(ticket_seq)
    .fill(&client)
    .await
    .expect("Failed to auto-fill payment");

    let base_fee: u64 = match payment_builder.fee() {
        Amount::Xrpl(s) => s.parse().expect("Fee is not a valid u64"),
        _ => unreachable!("fee is always XRP drops"),
    };
    let multisign_fee = Amount::Xrpl(((1 + 2) * base_fee).to_string());

    let mut payment = payment_builder
        .with_fee(multisign_fee)
        .build()
        .expect("Failed to build payment");

    let sig1 = payment.sign_as(&wallet1).expect("Failed to sign as wallet 1");
    let sig3 = payment.sign_as(&wallet3).expect("Failed to sign as wallet 3");
    payment.add_signature(sig1);
    payment.add_signature(sig3);

    let submit_request = SubmitMultisignedRequestBuilder::new(&payment)
        .build()
        .expect("Failed to serialize multi-signed ticket payment");
    let ms_result = client
        .request(&submit_request)
        .await
        .expect("Failed to submit multi-signed ticket payment")
        .result()
        .expect("Failed to get multi-signed ticket payment result");

    let code = &ms_result.engine_result;
    assert!(
        !code.starts_with("tem"),
        "multi-signed ticket payment malformed ({code})"
    );
    assert!(
        code.starts_with("tes"),
        "multi-signed ticket payment failed: engine_result = {code}"
    );

    while let Ok(msg) = stream.recv().await {
        if msg.tx_json.ticket_sequence == payment.ticket_sequence {
            assert_eq!(
                msg.engine_result, "tesSUCCESS",
                "multi-signed ticket payment not validated: {}",
                msg.engine_result
            );
            break;
        }
    }

    // --- Teardown: remove the signer list so account 2 reverts to single-key signing ---

    let remove = SignerListSetBuilder::new(account2.clone(), 0)
        .fill(&client)
        .await
        .expect("Failed to auto-fill SignerListSet removal")
        .build()
        .expect("Failed to build SignerListSet removal");

    let submit = SubmitRequestBuilder::new(&remove, &wallet2)
        .build()
        .expect("Failed to build submit request");

    client
        .request(&submit)
        .await
        .expect("Failed to submit SignerListSet removal");

    while let Ok(msg) = stream.recv().await {
        if msg.tx_json.sequence == remove.sequence {
            assert_eq!(
                msg.engine_result, "tesSUCCESS",
                "SignerList removal failed: {}",
                msg.engine_result
            );
            break;
        }
    }
}
