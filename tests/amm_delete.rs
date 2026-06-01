mod common;

use anyhow::Result;
use xrpl::{
    types::{
        Asset,
        builders::{AMMDeleteBuilder, SubmitRequestBuilder},
    },
    Client,
};
use common::{server_url, test_seed};
use ripple_keypairs::Seed;

#[test]
fn test_amm_delete_builder_simple() -> Result<()> {
    let asset = Asset::xrp();
    let asset2 = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
    let amm_delete = AMMDeleteBuilder::new(
        "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        asset.clone(),
        asset2.clone(),
    )
    .build()?;

    let tx_json = serde_json::to_value(&amm_delete)?;

    assert_eq!(tx_json["TransactionType"], "AMMDelete");
    assert_eq!(tx_json["Account"], "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    assert_eq!(tx_json["Asset"], serde_json::to_value(asset)?);
    assert_eq!(tx_json["Asset2"], serde_json::to_value(asset2)?);

    Ok(())
}

#[tokio::test]
async fn test_amm_delete_submit() -> Result<()> {
    let client = Client::new(server_url());

    let seed_str = test_seed(1);
    let seed: Seed = seed_str.parse()?;
    let (private_key, public_key) = seed.derive_keypair()?;
    let wallet = common::Wallet { public_key, private_key };
    let address = wallet.public_key.derive_address();

    let asset = Asset::xrp();
    let asset2 = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;

    let builder = AMMDeleteBuilder::new(address, asset, asset2);

    let filled_builder = builder.fill(&client).await?;

    let tx = filled_builder.build()?;

    let request =
        SubmitRequestBuilder::new(&tx, &wallet).fail_hard(true).build()?;

    // No XRP/USD AMM exists on testnet; the transaction must fail with terNO_AMM.
    let submit_response = client.request(request).await?.result()?;
    assert_eq!(submit_response.engine_result, "terNO_AMM");

    Ok(())
}
