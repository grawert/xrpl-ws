use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    transactions::xchain::XChainCreateClaimID, Amount, TransactionType,
    XChainBridge,
};

/// Builder for XRPL XChainCreateClaimID transactions.
///
/// Creates a new cross-chain claim ID that is used for a cross-chain transfer.
/// A claim ID must be created before a `XChainCommit` can be submitted on the
/// source chain.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # use xrpl::types::Amount;
/// use xrpl::{Client, xrp, types::{Asset, XChainBridge, builders::XChainCreateClaimIDBuilder}};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = XChainCreateClaimIDBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     xrp!(100),
///     XChainBridge {
///         locking_chain_door: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         locking_chain_issue: Asset::xrp(),
///         issuing_chain_door: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
///         issuing_chain_issue: Asset::xrp(),
///     },
/// )
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type XChainCreateClaimIDBuilder = TransactionBuilder<XChainCreateClaimID>;

impl XChainCreateClaimIDBuilder {
    /// Creates a new `XChainCreateClaimIDBuilder` with all required fields.
    pub fn new(
        account: impl AsRef<str>,
        other_chain_source: impl AsRef<str>,
        signature_reward: impl Into<Amount>,
        xchain_bridge: impl Into<XChainBridge>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            XChainCreateClaimID {
                other_chain_source: other_chain_source.as_ref().to_string(),
                signature_reward: signature_reward.into(),
                xchain_bridge: xchain_bridge.into(),
            },
        )
    }
}

impl TransactionTypeBuilder for XChainCreateClaimID {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::XChainCreateClaimID(self))
    }
}
