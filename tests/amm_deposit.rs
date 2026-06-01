mod common;

use anyhow::Result;
use xrpl::{
    types::{
        Asset, Amount, AMMDepositFlags,
        builders::{AMMDepositBuilder, SubmitRequestBuilder},
    },
    Client,
};
use common::{server_url, test_seed};
use ripple_keypairs::Seed;

/// XRP amount each test deposits into the AMM (passed as a decimal-XRP string).
const DEPOSIT_AMOUNT_XRP: &str = "10000000";

// SingleAsset mode: Amount only.
#[test]
fn test_amm_deposit_builder_single_asset() -> Result<()> {
    let asset = Asset::xrp();
    let asset2 = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
    let amount = Amount::xrp(DEPOSIT_AMOUNT_XRP)?;

    let amm_deposit = AMMDepositBuilder::new(
        "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        asset.clone(),
        asset2.clone(),
    )
    .with_amount(amount.clone())
    .build()?;

    let tx_json = serde_json::to_value(&amm_deposit)?;

    assert_eq!(tx_json["TransactionType"], "AMMDeposit");
    assert_eq!(tx_json["Account"], "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    assert_eq!(tx_json["Asset"], serde_json::to_value(asset)?);
    assert_eq!(tx_json["Asset2"], serde_json::to_value(asset2)?);
    assert_eq!(tx_json["Amount"], serde_json::to_value(amount)?);

    Ok(())
}

// TwoAsset mode: Amount + Amount2 (+ optional TradingFee vote).
#[test]
fn test_amm_deposit_builder_two_asset() -> Result<()> {
    let asset = Asset::xrp();
    let asset2 = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
    let amount = Amount::xrp(DEPOSIT_AMOUNT_XRP)?;
    let amount2 = Amount::issued_currency(
        "100",
        "USD",
        "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
    )?;

    let amm_deposit = AMMDepositBuilder::new(
        "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        asset.clone(),
        asset2.clone(),
    )
    .with_amount(amount.clone())
    .with_amount2(amount2.clone())
    .with_trading_fee(500)
    .build()?;

    let tx_json = serde_json::to_value(&amm_deposit)?;

    assert_eq!(tx_json["TransactionType"], "AMMDeposit");
    assert_eq!(tx_json["Amount"], serde_json::to_value(amount)?);
    assert_eq!(tx_json["Amount2"], serde_json::to_value(amount2)?);
    assert_eq!(tx_json["TradingFee"], 500);

    Ok(())
}

// LimitLPToken mode: Amount + EPrice.
#[test]
fn test_amm_deposit_builder_limit_lp_token() -> Result<()> {
    let asset = Asset::xrp();
    let asset2 = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
    let amount = Amount::xrp(DEPOSIT_AMOUNT_XRP)?;
    let e_price = Amount::xrp("100")?;

    let amm_deposit = AMMDepositBuilder::new(
        "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        asset.clone(),
        asset2.clone(),
    )
    .with_amount(amount.clone())
    .with_e_price(e_price.clone())
    .build()?;

    let tx_json = serde_json::to_value(&amm_deposit)?;

    assert_eq!(tx_json["TransactionType"], "AMMDeposit");
    assert_eq!(tx_json["Amount"], serde_json::to_value(amount)?);
    assert_eq!(tx_json["EPrice"], serde_json::to_value(e_price)?);

    Ok(())
}

// LPToken mode: LPTokenOut only.
#[test]
fn test_amm_deposit_builder_lp_token() -> Result<()> {
    let asset = Asset::xrp();
    let asset2 = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
    let lp_token_out = Amount::issued_currency(
        "100",
        "LPT",
        "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
    )?;

    let amm_deposit = AMMDepositBuilder::new(
        "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        asset.clone(),
        asset2.clone(),
    )
    .with_lp_token_out(lp_token_out.clone())
    .build()?;

    let tx_json = serde_json::to_value(&amm_deposit)?;

    assert_eq!(tx_json["TransactionType"], "AMMDeposit");
    assert_eq!(tx_json["LPTokenOut"], serde_json::to_value(lp_token_out)?);

    Ok(())
}

#[tokio::test]
async fn test_amm_deposit_submit() -> Result<()> {
    let client = Client::new(server_url());

    let seed_str = test_seed(1);
    let seed: Seed = seed_str.parse()?;
    let (private_key, public_key) = seed.derive_keypair()?;
    let wallet = common::Wallet { public_key, private_key };
    let address = wallet.public_key.derive_address();

    let asset = Asset::xrp();
    let asset2 = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;

    let tx = AMMDepositBuilder::new(address, asset, asset2)
        .with_amount(Amount::xrp(DEPOSIT_AMOUNT_XRP)?)
        .with_flags(AMMDepositFlags::SINGLE_ASSET)
        .fill(&client)
        .await?
        .build()?;

    let request =
        SubmitRequestBuilder::new(&tx, &wallet).fail_hard(true).build()?;

    // No XRP/USD AMM exists on testnet; the transaction must fail with terNO_AMM.
    // telINSUF_FEE_P is accepted as a transient testnet condition.
    let submit_response = client.request(request).await?.result()?;
    let code = &submit_response.engine_result;
    assert!(
        code == "terNO_AMM" || code == "telINSUF_FEE_P",
        "unexpected engine_result: {code}"
    );

    Ok(())
}
