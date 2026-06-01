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

/// Sets up a 2-of-2 signer list on account 2 (TEST_SEED_2), submits a multi-signed
/// payment to account 1 co-signed by accounts 1 and 3, then removes the signer list as teardown.
#[serial]
#[tokio::test]
async fn test_multi_sign_payment() {
    let seed1: Seed = test_seed(1).parse().unwrap();
    let (priv1, pub1) = seed1.derive_keypair().unwrap();
    let wallet1 = Wallet { public_key: pub1, private_key: priv1 };
    let account1 = wallet1.public_key.derive_address();

    let seed2: Seed = test_seed(2).parse().unwrap();
    let (priv2, pub2) = seed2.derive_keypair().unwrap();
    let wallet2 = Wallet { public_key: pub2, private_key: priv2 };
    let account2 = wallet2.public_key.derive_address();

    let seed3: Seed = test_seed(3).parse().unwrap();
    let (priv3, pub3) = seed3.derive_keypair().unwrap();
    let wallet3 = Wallet { public_key: pub3, private_key: priv3 };
    let account3 = wallet3.public_key.derive_address();

    let client = Client::new(server_url());

    let sub_req =
        AccountTransactionsSubscription::validated(vec![account2.clone()])
            .expect("Valid address");
    let (_resp, mut handle) =
        client.subscribe(sub_req).await.expect("Subscription failed");

    // --- Step 1: Set up a 2-of-2 signer list on account 2 ---

    let signer_list_tx = SignerListSetBuilder::new(account2.clone(), 2)
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
        .unwrap()
        .build()
        .unwrap();

    client
        .request(
            SubmitRequestBuilder::new(&signer_list_tx, &wallet2)
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

    while let Ok(msg) = handle.recv().await {
        if msg.tx_json.sequence == signer_list_tx.sequence {
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
    // Multi-signing fee: (1 + number_of_signers) × base_fee.
    // We fill() to get the current open_ledger_fee, then multiply by (1 + 2 signers).

    let payment_builder = PaymentBuilder::new(
        account2.clone(),
        account1.clone(),
        xrp!(PAYMENT_AMOUNT_XRP),
    )
    .fill(&client)
    .await
    .unwrap();

    let base_fee: u64 = match payment_builder.fee() {
        Amount::Xrpl(s) => s.parse().unwrap(),
        _ => unreachable!("fee is always XRP drops"),
    };
    let multisign_fee = Amount::Xrpl(((1 + 2) * base_fee).to_string());

    let mut payment_tx =
        payment_builder.with_fee(multisign_fee).build().unwrap();

    let sig1 = payment_tx.sign_as(&wallet1).unwrap();
    let sig3 = payment_tx.sign_as(&wallet3).unwrap();
    payment_tx.add_signature(sig1);
    payment_tx.add_signature(sig3);

    let ms_result = client
        .request(SubmitMultisignedRequestBuilder::new(&payment_tx).build())
        .await
        .unwrap()
        .result()
        .unwrap();

    let code = &ms_result.engine_result;
    assert!(
        !code.starts_with("tem"),
        "multi-signed payment malformed ({code})"
    );
    assert!(
        code.starts_with("tes"),
        "multi-signed payment failed: engine_result = {code}"
    );

    while let Ok(msg) = handle.recv().await {
        if msg.tx_json.sequence == payment_tx.sequence {
            assert_eq!(
                msg.engine_result, "tesSUCCESS",
                "multi-signed payment not validated: {}",
                msg.engine_result
            );
            break;
        }
    }

    // --- Teardown: remove the signer list so account 2 reverts to single-key signing ---

    let remove_tx = SignerListSetBuilder::new(account2.clone(), 0)
        .fill(&client)
        .await
        .unwrap()
        .build()
        .unwrap();

    client
        .request(
            SubmitRequestBuilder::new(&remove_tx, &wallet2).build().unwrap(),
        )
        .await
        .unwrap();

    while let Ok(msg) = handle.recv().await {
        if msg.tx_json.sequence == remove_tx.sequence {
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
    let seed1: Seed = test_seed(1).parse().unwrap();
    let (priv1, pub1) = seed1.derive_keypair().unwrap();
    let wallet1 = Wallet { public_key: pub1, private_key: priv1 };
    let account1 = wallet1.public_key.derive_address();

    let seed2: Seed = test_seed(2).parse().unwrap();
    let (priv2, pub2) = seed2.derive_keypair().unwrap();
    let wallet2 = Wallet { public_key: pub2, private_key: priv2 };
    let account2 = wallet2.public_key.derive_address();

    let seed3: Seed = test_seed(3).parse().unwrap();
    let (priv3, pub3) = seed3.derive_keypair().unwrap();
    let wallet3 = Wallet { public_key: pub3, private_key: priv3 };
    let account3 = wallet3.public_key.derive_address();

    let client = Client::new(server_url());

    let sub_req =
        AccountTransactionsSubscription::validated(vec![account2.clone()])
            .expect("Valid address");
    let (_resp, mut handle) =
        client.subscribe(sub_req).await.expect("Subscription failed");

    // --- Step 1: Set up a 2-of-2 signer list on account 2 ---

    let signer_list_tx = SignerListSetBuilder::new(account2.clone(), 2)
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
        .unwrap()
        .build()
        .unwrap();

    client
        .request(
            SubmitRequestBuilder::new(&signer_list_tx, &wallet2)
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

    while let Ok(msg) = handle.recv().await {
        if msg.tx_json.sequence == signer_list_tx.sequence {
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
        .unwrap()
        .build()
        .unwrap();

    let ticket_create_seq = ticket_create_tx.sequence;
    let ticket_seq = ticket_create_tx.ticket_sequences().unwrap()[0];

    client
        .request(
            SubmitRequestBuilder::new(&ticket_create_tx, &wallet2)
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

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

    // --- Step 3: Build and submit a multi-signed payment using the ticket ---
    //
    // with_ticket_sequence before fill() skips the account_info round-trip and
    // sets Sequence = 0 automatically. The multi-sign fee is still (1 + N) × base_fee.

    let payment_builder = PaymentBuilder::new(
        account2.clone(),
        account1.clone(),
        xrp!(PAYMENT_AMOUNT_XRP),
    )
    .with_ticket_sequence(ticket_seq)
    .fill(&client)
    .await
    .unwrap();

    let base_fee: u64 = match payment_builder.fee() {
        Amount::Xrpl(s) => s.parse().unwrap(),
        _ => unreachable!("fee is always XRP drops"),
    };
    let multisign_fee = Amount::Xrpl(((1 + 2) * base_fee).to_string());

    let mut payment_tx =
        payment_builder.with_fee(multisign_fee).build().unwrap();

    let sig1 = payment_tx.sign_as(&wallet1).unwrap();
    let sig3 = payment_tx.sign_as(&wallet3).unwrap();
    payment_tx.add_signature(sig1);
    payment_tx.add_signature(sig3);

    let ms_result = client
        .request(SubmitMultisignedRequestBuilder::new(&payment_tx).build())
        .await
        .unwrap()
        .result()
        .unwrap();

    let code = &ms_result.engine_result;
    assert!(
        !code.starts_with("tem"),
        "multi-signed ticket payment malformed ({code})"
    );
    assert!(
        code.starts_with("tes"),
        "multi-signed ticket payment failed: engine_result = {code}"
    );

    while let Ok(msg) = handle.recv().await {
        if msg.tx_json.ticket_sequence == payment_tx.ticket_sequence {
            assert_eq!(
                msg.engine_result, "tesSUCCESS",
                "multi-signed ticket payment not validated: {}",
                msg.engine_result
            );
            break;
        }
    }

    // --- Teardown: remove the signer list so account 2 reverts to single-key signing ---

    let remove_tx = SignerListSetBuilder::new(account2.clone(), 0)
        .fill(&client)
        .await
        .unwrap()
        .build()
        .unwrap();

    client
        .request(
            SubmitRequestBuilder::new(&remove_tx, &wallet2).build().unwrap(),
        )
        .await
        .unwrap();

    while let Ok(msg) = handle.recv().await {
        if msg.tx_json.sequence == remove_tx.sequence {
            assert_eq!(
                msg.engine_result, "tesSUCCESS",
                "SignerList removal failed: {}",
                msg.engine_result
            );
            break;
        }
    }
}
