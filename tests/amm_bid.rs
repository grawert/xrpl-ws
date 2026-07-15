mod common;

use anyhow::Result;
use xrpl::{
    types::{
        Asset, Amount,
        builders::{AMMBidBuilder, SubmitRequestBuilder},
    },
    Client,
};
use common::{sender_address, sender_wallet, server_url};

#[test]
fn test_amm_bid_builder_simple() -> Result<()> {
    let asset = Asset::xrp();
    let asset2 = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;

    let amm_bid =
        AMMBidBuilder::new("rPT0Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", asset, asset2)
            .build()?;

    let tx_json = serde_json::to_value(&amm_bid)?;

    assert_eq!(tx_json["TransactionType"], "AMMBid");
    assert_eq!(tx_json["Account"], "rPT0Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");

    Ok(())
}

#[test]
fn test_amm_bid_builder_with_bids() -> Result<()> {
    let asset = Asset::xrp();
    let asset2 = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
    let bid_min = Amount::xrp("10")?;
    let bid_max = Amount::xrp("20")?;

    let amm_bid =
        AMMBidBuilder::new("rPT0Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", asset, asset2)
            .with_bid_min(bid_min.clone())
            .with_bid_max(bid_max.clone())
            .build()?;

    let tx_json = serde_json::to_value(&amm_bid)?;

    assert_eq!(tx_json["TransactionType"], "AMMBid");
    assert_eq!(tx_json["Account"], "rPT0Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
    assert_eq!(tx_json["BidMin"], serde_json::to_value(bid_min)?);
    assert_eq!(tx_json["BidMax"], serde_json::to_value(bid_max)?);

    Ok(())
}

#[test]
fn test_amm_bid_builder_with_bid_range() -> Result<()> {
    let asset = Asset::xrp();
    let asset2 = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
    let bid_min = Amount::xrp("10")?;
    let bid_max = Amount::xrp("20")?;

    let amm_bid =
        AMMBidBuilder::new("rPT0Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", asset, asset2)
            .with_bid_range(bid_min.clone(), bid_max.clone())
            .build()?;

    let tx_json = serde_json::to_value(&amm_bid)?;

    assert_eq!(tx_json["TransactionType"], "AMMBid");
    assert_eq!(tx_json["Account"], "rPT0Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
    assert_eq!(tx_json["BidMin"], serde_json::to_value(bid_min)?);
    assert_eq!(tx_json["BidMax"], serde_json::to_value(bid_max)?);

    Ok(())
}

#[test]
fn test_amm_bid_builder_with_auth_accounts() -> Result<()> {
    let asset = Asset::xrp();
    let asset2 = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
    let accounts = [
        "rPT0Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
    ];

    let amm_bid =
        AMMBidBuilder::new("rPT0Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", asset, asset2)
            .with_auth_accounts(accounts)
            .build()?;

    let tx_json = serde_json::to_value(&amm_bid)?;

    assert_eq!(tx_json["TransactionType"], "AMMBid");
    assert_eq!(tx_json["Account"], "rPT0Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
    assert!(tx_json["AuthAccounts"].is_array());
    assert_eq!(tx_json["AuthAccounts"].as_array().unwrap().len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_amm_bid_submit() -> Result<()> {
    let client = Client::new(server_url());

    let wallet = sender_wallet();
    let address = sender_address();

    let asset = Asset::xrp();
    let asset2 = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;

    let builder = AMMBidBuilder::new(address, asset, asset2);

    let filled_builder = builder.fill(&client).await?;

    let tx = filled_builder.build()?;

    let request =
        SubmitRequestBuilder::new(&tx, &wallet).fail_hard(true).build()?;

    // No XRP/USD AMM exists on testnet; the transaction must fail with terNO_AMM.
    let submit_response = client.request(&request).await?.result()?;
    assert_eq!(submit_response.engine_result, "terNO_AMM");

    Ok(())
}
