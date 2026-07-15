mod common;

use anyhow::Result;
use xrpl::{
    types::{
        Amount,
        builders::{AMMCreateBuilder, SubmitRequestBuilder},
    },
    Client,
};
use common::{sender_address, sender_wallet, server_url};

/// Initial XRP liquidity seeded into the AMM pool by each test (decimal-XRP string).
const INITIAL_LIQUIDITY_XRP: &str = "50000000";

#[test]
fn test_amm_create_builder_simple() -> Result<()> {
    let amount = Amount::xrp(INITIAL_LIQUIDITY_XRP)?;
    let amount2 = Amount::issued_currency(
        "500",
        "USD",
        "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
    )?;
    let amm_create = AMMCreateBuilder::new(
        "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        amount.clone(),
        amount2.clone(),
        500,
    )
    .build()?;

    let tx_json = serde_json::to_value(&amm_create)?;

    assert_eq!(tx_json["TransactionType"], "AMMCreate");
    assert_eq!(tx_json["Account"], "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    assert_eq!(tx_json["Amount"], serde_json::to_value(amount)?);
    assert_eq!(tx_json["Amount2"], serde_json::to_value(amount2)?);
    assert_eq!(tx_json["TradingFee"], 500);

    Ok(())
}

#[tokio::test]
async fn test_amm_create_submit() -> Result<()> {
    let client = Client::new(server_url());

    let wallet = sender_wallet();
    let address = sender_address();

    let amount = Amount::xrp(INITIAL_LIQUIDITY_XRP)?;
    let amount2 = Amount::issued_currency(
        "500",
        "USD",
        "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
    )?;

    let builder = AMMCreateBuilder::new(address, amount, amount2, 500);

    let filled_builder = builder.fill(&client).await?;

    let tx = filled_builder.build()?;

    let request =
        SubmitRequestBuilder::new(&tx, &wallet).fail_hard(true).build()?;

    // Expected: a tec code because the test account has no USD trust line /
    // balance for the genesis issuer. telINSUF_FEE_P is allowed as a transient
    // testnet condition when the fee level rises between fill() and submit.
    let submit_response = client.request(&request).await?.result()?;
    let code = &submit_response.engine_result;
    assert!(
        code.starts_with("tec") || code == "telINSUF_FEE_P",
        "unexpected engine_result: {code}"
    );

    Ok(())
}
