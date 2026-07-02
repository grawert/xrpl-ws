mod common;

use std::time::Duration;
use xrpl_mithril::types::AccountId;
use ripple_keypairs::Seed;
use serial_test::serial;
use tokio::time::timeout;
use xrpl::subscriptions::{
    AccountTransactionsSubscription, AccountTransactionMessage,
};
use xrpl::types::Amount;
use xrpl::types::MPTokenAuthorizeFlags as AuthFlags;
use xrpl::types::MPTokenIssuanceCreateFlags as IssuanceFlags;
use xrpl::types::builders::ClawbackBuilder;
use xrpl::types::builders::MPTokenIssuanceCreateBuilder;
use xrpl::types::builders::MPTokenIssuanceDestroyBuilder;
use xrpl::types::builders::MPTokenAuthorizeBuilder;
use xrpl::types::builders::PaymentBuilder;
use xrpl::types::builders::SubmitRequestBuilder;
use xrpl::{Client, SubscriptionStream};
use common::*;

const VALIDATION_TIMEOUT: Duration = Duration::from_secs(30);

/// Computes the MPTokenIssuanceID from the issuer address and the sequence
/// number of the MPTokenIssuanceCreate transaction.
///
/// Format: 4-byte big-endian sequence || 20-byte account ID = 48 hex chars.
fn mpt_issuance_id(issuer_address: &str, sequence: u32) -> String {
    let account_id: AccountId =
        issuer_address.parse().expect("Failed to decode issuer address");
    let mut id = [0u8; 24];
    id[..4].copy_from_slice(&sequence.to_be_bytes());
    id[4..].copy_from_slice(account_id.as_ref());
    hex::encode(id).to_uppercase()
}

async fn wait_for_sequence(
    stream: &mut SubscriptionStream<AccountTransactionMessage>,
    sequence: u32,
    context: &str,
) {
    loop {
        let msg = timeout(VALIDATION_TIMEOUT, stream.recv())
            .await
            .unwrap_or_else(|_| {
                panic!("{context}: timed out after {}s waiting for seq={sequence} to validate",
                    VALIDATION_TIMEOUT.as_secs())
            })
            .expect("Subscription channel closed unexpectedly");

        if msg.tx_json.sequence == sequence {
            assert_eq!(
                msg.engine_result, "tesSUCCESS",
                "{context}: seq={sequence} failed: {}",
                msg.engine_result
            );
            return;
        }
    }
}

/// MPToken authorized trust line lifecycle (RWA equity token, 2% transfer fee):
/// 1. Issuer creates an MPTokenIssuance with REQUIRE_AUTH | CAN_TRANSFER and 2% fee.
/// 2. Holder opts in via MPTokenAuthorize (creates the MPToken slot).
/// 3. Issuer grants authorization via MPTokenAuthorize with Holder field set.
/// 4. Issuer sends tokens to holder via Payment.
/// 5. Holder returns tokens to issuer, draining the outstanding supply.
/// 6. Holder opts out via MPTokenAuthorize with tfMPTUnauthorize.
/// 7. Issuer destroys the issuance.

#[serial]
#[tokio::test]
async fn test_mpt_rwa_authorized_issuance() {
    let issuer_seed: Seed = test_seed(1).parse().unwrap();
    let (issuer_priv, issuer_pub) = issuer_seed.derive_keypair().unwrap();
    let issuer_wallet =
        Wallet { public_key: issuer_pub, private_key: issuer_priv };
    let issuer_addr = issuer_wallet.public_key.derive_address();

    let holder_seed: Seed =
        test_seed(2).parse().expect("Failed to parse holder seed");
    let (holder_priv, holder_pub) =
        holder_seed.derive_keypair().expect("Failed to derive holder keypair");
    let holder_wallet =
        Wallet { public_key: holder_pub, private_key: holder_priv };
    let holder_addr = holder_wallet.public_key.derive_address();

    let client = Client::new(server_url());

    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");

    let (_, mut issuer_handle) = conn
        .subscribe(
            &AccountTransactionsSubscription::validated([issuer_addr.clone()])
                .expect("Failed to create issuer subscription"),
        )
        .await
        .expect("issuer subscribe failed");

    let (_, mut holder_handle) = conn
        .subscribe(
            &AccountTransactionsSubscription::validated(vec![
                holder_addr.clone(),
            ])
            .expect("Failed to create holder subscription"),
        )
        .await
        .expect("holder subscribe failed");

    const TRANSFER_FEE: u16 = 2000; // 2% (units of 1/1000 of a percent)
    const TRANSFER_AMOUNT: &str = "1000"; // 1000 tokens

    // Step 1: Issuer creates the MPTokenIssuance

    // XLS-89 compressed-key metadata schema
    let metadata = concat!(
        r#"{"t":"RWAX","#,
        r#""n":"RWAX","#,
        r#""d":"Tokenized equity with authorized holder access.","#,
        r#""i":"https://raw.githubusercontent.com/grawert/xrpl-ws/refs/heads/main/resources/rwax.svg","#,
        r#""ac":"rwa","#,
        r#""as":"equity","#,
        r#""in":"xrpl-ws","#,
        r#""us":["#,
            r#"{"u":"https://crates.io/crates/xrpl-ws","#,
            r#""c":"website","#,
            r#""t":"Lightweight async WebSocket client for the XRP Ledger"}"#,
        r#"],"#,
        r#""ai":{"restricted":true}}"#,
    )
    .as_bytes();
    let metadata_hex = hex::encode(metadata).to_uppercase();

    let create_tx = MPTokenIssuanceCreateBuilder::new(issuer_addr.clone())
        .with_transfer_fee(TRANSFER_FEE)
        .with_mpt_metadata(metadata_hex)
        .with_flags(IssuanceFlags::REQUIRE_AUTH | IssuanceFlags::CAN_TRANSFER)
        .fill(&client)
        .await
        .expect("Failed to auto-fill create_tx")
        .build()
        .expect("Failed to build create_tx");

    let result = client
        .request(
            &SubmitRequestBuilder::new(&create_tx, &issuer_wallet)
                .build()
                .expect("Failed to build submit request"),
        )
        .await
        .expect("create submit failed")
        .result()
        .expect("create result failed");

    assert_accepted(&result, "MPTokenIssuanceCreate");
    wait_for_sequence(
        &mut issuer_handle,
        create_tx.sequence,
        "MPTokenIssuanceCreate",
    )
    .await;

    let issuance_id = mpt_issuance_id(&issuer_addr, create_tx.sequence);

    // Step 2: Holder opts in (creates the MPToken slot; not yet authorized)
    let opt_in_tx =
        MPTokenAuthorizeBuilder::new(holder_addr.clone(), issuance_id.clone())
            .fill(&client)
            .await
            .expect("Failed to auto-fill opt-in tx")
            .build()
            .expect("Failed to build opt-in tx");

    let result = client
        .request(
            &SubmitRequestBuilder::new(&opt_in_tx, &holder_wallet)
                .build()
                .expect("Failed to build submit request"),
        )
        .await
        .expect("opt-in submit failed")
        .result()
        .expect("opt-in result failed");

    assert_accepted(&result, "MPTokenAuthorize opt-in");
    wait_for_sequence(
        &mut holder_handle,
        opt_in_tx.sequence,
        "MPTokenAuthorize opt-in",
    )
    .await;

    // Step 3: Issuer grants authorization to the holder
    let authorize_tx =
        MPTokenAuthorizeBuilder::new(issuer_addr.clone(), issuance_id.clone())
            .with_holder(holder_addr.clone())
            .fill(&client)
            .await
            .expect("Failed to auto-fill authorize tx")
            .build()
            .expect("Failed to build authorize tx");

    let result = client
        .request(
            &SubmitRequestBuilder::new(&authorize_tx, &issuer_wallet)
                .build()
                .expect("Failed to build submit request"),
        )
        .await
        .expect("issuer authorize submit failed")
        .result()
        .expect("issuer authorize result failed");

    assert_accepted(&result, "MPTokenAuthorize issuer→holder");
    wait_for_sequence(
        &mut issuer_handle,
        authorize_tx.sequence,
        "MPTokenAuthorize issuer→holder",
    )
    .await;

    // Step 4: Issuer sends tokens to holder
    let send_tx = PaymentBuilder::new(
        issuer_addr.clone(),
        holder_addr.clone(),
        Amount::mpt(TRANSFER_AMOUNT, &issuance_id)
            .expect("Failed to create MPT amount"),
    )
    .fill(&client)
    .await
    .expect("Failed to auto-fill send tx")
    .build()
    .expect("Failed to build send tx");

    let result = client
        .request(
            &SubmitRequestBuilder::new(&send_tx, &issuer_wallet)
                .build()
                .expect("Failed to build submit request"),
        )
        .await
        .expect("payment submit failed")
        .result()
        .expect("payment result failed");

    assert_accepted(&result, "RWAX Payment issuer→holder");
    wait_for_sequence(
        &mut issuer_handle,
        send_tx.sequence,
        "RWAX Payment issuer→holder",
    )
    .await;

    // Step 5: Holder returns tokens to issuer (drains outstanding supply)
    let return_tx = PaymentBuilder::new(
        holder_addr.clone(),
        issuer_addr.clone(),
        Amount::mpt(TRANSFER_AMOUNT, &issuance_id)
            .expect("Failed to create MPT amount"),
    )
    .fill(&client)
    .await
    .expect("Failed to auto-fill return tx")
    .build()
    .expect("Failed to build return tx");

    let result = client
        .request(
            &SubmitRequestBuilder::new(&return_tx, &holder_wallet)
                .build()
                .expect("Failed to build submit request"),
        )
        .await
        .expect("return submit failed")
        .result()
        .expect("return result failed");

    assert_accepted(&result, "RWAX Payment holder→issuer");
    wait_for_sequence(
        &mut holder_handle,
        return_tx.sequence,
        "RWAX Payment holder→issuer",
    )
    .await;

    // Step 6: Holder opts out
    let opt_out_tx =
        MPTokenAuthorizeBuilder::new(holder_addr, issuance_id.clone())
            .with_flags(AuthFlags::UNAUTHORIZE)
            .fill(&client)
            .await
            .expect("Failed to auto-fill opt-out tx")
            .build()
            .expect("Failed to build opt-out tx");

    let result = client
        .request(
            &SubmitRequestBuilder::new(&opt_out_tx, &holder_wallet)
                .build()
                .expect("Failed to build submit request"),
        )
        .await
        .expect("opt-out submit failed")
        .result()
        .expect("opt-out result failed");

    assert_accepted(&result, "MPTokenAuthorize opt-out");
    wait_for_sequence(
        &mut holder_handle,
        opt_out_tx.sequence,
        "MPTokenAuthorize opt-out",
    )
    .await;

    // Step 7: Issuer destroys the issuance (outstanding amount is zero)
    let destroy_tx =
        MPTokenIssuanceDestroyBuilder::new(issuer_addr, issuance_id)
            .fill(&client)
            .await
            .expect("Failed to auto-fill destroy tx")
            .build()
            .expect("Failed to build destroy tx");

    let result = client
        .request(
            &SubmitRequestBuilder::new(&destroy_tx, &issuer_wallet)
                .build()
                .expect("Failed to build submit request"),
        )
        .await
        .expect("destroy submit failed")
        .result()
        .expect("destroy result failed");

    assert_accepted(&result, "MPTokenIssuanceDestroy");
    wait_for_sequence(
        &mut issuer_handle,
        destroy_tx.sequence,
        "MPTokenIssuanceDestroy",
    )
    .await;
}

/// USDx stablecoin simulation:
/// 1. Issuer creates a regulated MPTokenIssuance (AssetScale 2, 100M USDx cap, JSON metadata,
///    flags: CAN_LOCK | CAN_ESCROW | CAN_TRADE | CAN_TRANSFER | CAN_CLAWBACK).
/// 2. Holder opts in via MPTokenAuthorize.
/// 3. Issuer sends 49.99 USDx to holder via Payment.
/// 4. Issuer claws back 10.00 USDx from holder.
/// 5. Holder returns remaining 39.99 USDx to issuer, draining the outstanding supply.
/// 6. Holder opts out.
/// 7. Issuer destroys the issuance (only possible when outstanding balance is zero).
#[serial]
#[tokio::test]
async fn test_mpt_stablecoin_usdx_issuance() {
    let issuer_seed: Seed =
        test_seed(1).parse().expect("Failed to parse issuer seed");
    let (issuer_priv, issuer_pub) =
        issuer_seed.derive_keypair().expect("Failed to derive issuer keypair");
    let issuer_wallet =
        Wallet { public_key: issuer_pub, private_key: issuer_priv };
    let issuer_addr = issuer_wallet.public_key.derive_address();

    let holder_seed: Seed =
        test_seed(2).parse().expect("Failed to parse holder seed");
    let (holder_priv, holder_pub) =
        holder_seed.derive_keypair().expect("Failed to derive holder keypair");
    let holder_wallet =
        Wallet { public_key: holder_pub, private_key: holder_priv };
    let holder_addr = holder_wallet.public_key.derive_address();

    let client = Client::new(server_url());

    let mut conn = client
        .subscription()
        .await
        .expect("Failed to open subscription connection");

    let (_, mut issuer_handle) = conn
        .subscribe(
            &AccountTransactionsSubscription::validated([issuer_addr.clone()])
                .expect("Failed to create issuer subscription"),
        )
        .await
        .expect("issuer subscribe failed");

    let (_, mut holder_handle) = conn
        .subscribe(
            &AccountTransactionsSubscription::validated(vec![
                holder_addr.clone(),
            ])
            .expect("Failed to create holder subscription"),
        )
        .await
        .expect("holder subscribe failed");

    const MAX_SUPPLY: u64 = 10_000_000_000; // 100_000_000.00 USDx (AssetScale 2)
    const TRANSFER_AMOUNT: &str = "4999"; // 49.99 USDx (AssetScale 2)
    const CLAWBACK_AMOUNT: &str = "1000"; // 10.00 USDx (AssetScale 2)
    const RETURN_AMOUNT: &str = "3999"; // 39.99 USDx remaining after clawback

    // Step 1: Issuer creates the USDx issuance

    // XLS-89 compressed-key metadata schema
    let metadata = concat!(
        r#"{"t":"USDX","#,
        r#""n":"USDx","#,
        r#""d":"Stablecoin fake of USD.","#,
        r#""i":"https://raw.githubusercontent.com/grawert/xrpl-ws/refs/heads/main/resources/usdx.svg","#,
        r#""ac":"stablecoin","#,
        r#""as":"stablecoin","#,
        r#""in":"xrpl-ws","#,
        r#""us":["#,
            r#"{"u":"https://crates.io/crates/xrpl-ws","#,
            r#""c":"website","#,
            r#""t":"Lightweight async WebSocket client for the XRP Ledger"}"#,
        r#"]}"#,
    )
    .as_bytes();
    let metadata_hex = hex::encode(metadata).to_uppercase();

    let create_tx = MPTokenIssuanceCreateBuilder::new(issuer_addr.clone())
        .with_asset_scale(2)
        .with_maximum_amount(MAX_SUPPLY)
        .with_mpt_metadata(metadata_hex)
        .with_flags(
            IssuanceFlags::CAN_LOCK
                | IssuanceFlags::CAN_ESCROW
                | IssuanceFlags::CAN_TRADE
                | IssuanceFlags::CAN_TRANSFER
                | IssuanceFlags::CAN_CLAWBACK,
        )
        .fill(&client)
        .await
        .expect("Failed to auto-fill create tx")
        .build()
        .expect("Failed to build create tx");

    let result = client
        .request(
            &SubmitRequestBuilder::new(&create_tx, &issuer_wallet)
                .build()
                .expect("Failed to build submit request"),
        )
        .await
        .expect("create submit failed")
        .result()
        .expect("create result failed");

    assert_accepted(&result, "USDx MPTokenIssuanceCreate");
    wait_for_sequence(
        &mut issuer_handle,
        create_tx.sequence,
        "USDx MPTokenIssuanceCreate",
    )
    .await;

    let issuance_id = mpt_issuance_id(&issuer_addr, create_tx.sequence);

    // Step 2: Holder opts in
    let opt_in_tx =
        MPTokenAuthorizeBuilder::new(holder_addr.clone(), issuance_id.clone())
            .fill(&client)
            .await
            .expect("Failed to auto-fill opt-in tx")
            .build()
            .expect("Failed to build opt-in tx");

    let result = client
        .request(
            &SubmitRequestBuilder::new(&opt_in_tx, &holder_wallet)
                .build()
                .expect("Failed to build submit request"),
        )
        .await
        .expect("opt-in submit failed")
        .result()
        .expect("opt-in result failed");

    assert_accepted(&result, "USDx MPTokenAuthorize opt-in");
    wait_for_sequence(
        &mut holder_handle,
        opt_in_tx.sequence,
        "USDx MPTokenAuthorize opt-in",
    )
    .await;

    // Step 3: Issuer sends USDx to holder
    let send_tx = PaymentBuilder::new(
        issuer_addr.clone(),
        holder_addr.clone(),
        Amount::mpt(TRANSFER_AMOUNT, &issuance_id)
            .expect("Failed to create MPT amount"),
    )
    .fill(&client)
    .await
    .expect("Failed to auto-fill send tx")
    .build()
    .expect("Failed to build send tx");

    let result = client
        .request(
            &SubmitRequestBuilder::new(&send_tx, &issuer_wallet)
                .build()
                .expect("Failed to build submit request"),
        )
        .await
        .expect("payment submit failed")
        .result()
        .expect("payment result failed");

    assert_accepted(&result, "USDx Payment issuer→holder");
    wait_for_sequence(
        &mut issuer_handle,
        send_tx.sequence,
        "USDx Payment issuer→holder",
    )
    .await;

    // Step 4: Issuer claws back 10.00 USDx from holder
    let clawback_tx = ClawbackBuilder::new(
        issuer_addr.clone(),
        Amount::mpt(CLAWBACK_AMOUNT, &issuance_id)
            .expect("Failed to create MPT amount"),
    )
    .with_holder(holder_addr.clone())
    .fill(&client)
    .await
    .expect("Failed to auto-fill clawback tx")
    .build()
    .expect("Failed to build clawback tx");

    let result = client
        .request(
            &SubmitRequestBuilder::new(&clawback_tx, &issuer_wallet)
                .build()
                .expect("Failed to build submit request"),
        )
        .await
        .expect("clawback submit failed")
        .result()
        .expect("clawback result failed");

    assert_accepted(&result, "USDx Clawback issuer←holder");
    wait_for_sequence(
        &mut issuer_handle,
        clawback_tx.sequence,
        "USDx Clawback issuer←holder",
    )
    .await;

    // Step 5: Holder returns remaining 39.99 USDx to issuer (drains outstanding supply)
    let return_tx = PaymentBuilder::new(
        holder_addr.clone(),
        issuer_addr.clone(),
        Amount::mpt(RETURN_AMOUNT, &issuance_id)
            .expect("Failed to create MPT amount"),
    )
    .fill(&client)
    .await
    .expect("Failed to auto-fill return tx")
    .build()
    .expect("Failed to build return tx");

    let result = client
        .request(
            &SubmitRequestBuilder::new(&return_tx, &holder_wallet)
                .build()
                .expect("Failed to build submit request"),
        )
        .await
        .expect("return submit failed")
        .result()
        .expect("return result failed");

    assert_accepted(&result, "USDx Payment holder→issuer");
    wait_for_sequence(
        &mut holder_handle,
        return_tx.sequence,
        "USDx Payment holder→issuer",
    )
    .await;

    // Step 6: Holder opts out
    let opt_out_tx =
        MPTokenAuthorizeBuilder::new(holder_addr, issuance_id.clone())
            .with_flags(AuthFlags::UNAUTHORIZE)
            .fill(&client)
            .await
            .expect("Failed to auto-fill opt-out tx")
            .build()
            .expect("Failed to build opt-out tx");

    let result = client
        .request(
            &SubmitRequestBuilder::new(&opt_out_tx, &holder_wallet)
                .build()
                .expect("Failed to build submit request"),
        )
        .await
        .expect("opt-out submit failed")
        .result()
        .expect("opt-out result failed");

    assert_accepted(&result, "USDx MPTokenAuthorize opt-out");
    wait_for_sequence(
        &mut holder_handle,
        opt_out_tx.sequence,
        "USDx MPTokenAuthorize opt-out",
    )
    .await;

    // Step 7: Issuer destroys the issuance
    let destroy_tx =
        MPTokenIssuanceDestroyBuilder::new(issuer_addr, issuance_id)
            .fill(&client)
            .await
            .expect("Failed to auto-fill destroy tx")
            .build()
            .expect("Failed to build destroy tx");

    let result = client
        .request(
            &SubmitRequestBuilder::new(&destroy_tx, &issuer_wallet)
                .build()
                .expect("Failed to build submit request"),
        )
        .await
        .expect("destroy submit failed")
        .result()
        .expect("destroy result failed");

    assert_accepted(&result, "USDx MPTokenIssuanceDestroy");
    wait_for_sequence(
        &mut issuer_handle,
        destroy_tx.sequence,
        "USDx MPTokenIssuanceDestroy",
    )
    .await;
}
