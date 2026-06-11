use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    transactions::xchain::XChainModifyBridge, Amount, TransactionType,
    XChainBridge,
};

/// Builder for XRPL XChainModifyBridge transactions.
///
/// Modifies the parameters of an existing cross-chain bridge.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # use xrpl::types::Amount;
/// use xrpl::{Client, xrp, types::{Asset, XChainBridge, builders::XChainModifyBridgeBuilder}};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = XChainModifyBridgeBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     XChainBridge {
///         locking_chain_door: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         locking_chain_issue: Asset::xrp(),
///         issuing_chain_door: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
///         issuing_chain_issue: Asset::xrp(),
///     },
/// )
/// .with_signature_reward(xrp!(200))
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type XChainModifyBridgeBuilder = TransactionBuilder<XChainModifyBridge>;

impl XChainModifyBridgeBuilder {
    /// Creates a new `XChainModifyBridgeBuilder`; set at least one optional field before building.
    pub fn new(
        account: impl AsRef<str>,
        xchain_bridge: impl Into<XChainBridge>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            XChainModifyBridge {
                signature_reward: None,
                min_account_create_amount: None,
                xchain_bridge: xchain_bridge.into(),
            },
        )
    }

    /// Sets the new total reward paid to witness servers per attestation batch.
    pub fn with_signature_reward(
        mut self,
        signature_reward: impl Into<Amount>,
    ) -> Self {
        self.transaction_type.signature_reward = Some(signature_reward.into());
        self
    }

    /// Sets the new minimum XRP required to create an account on the issuing chain.
    pub fn with_min_account_create_amount(
        mut self,
        amount: impl Into<Amount>,
    ) -> Self {
        self.transaction_type.min_account_create_amount = Some(amount.into());
        self
    }
}

impl TransactionTypeBuilder for XChainModifyBridge {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::XChainModifyBridge(self))
    }
}
