use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    transactions::xchain::XChainCreateBridge, validation::validate_amount,
    Amount, TransactionType, XChainBridge,
};

/// Builder for XRPL XChainCreateBridge transactions.
///
/// Creates a new cross-chain bridge ledger object. Must be submitted to both
/// the locking chain and the issuing chain.
///
/// # Example
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # use xrpl::types::Amount;
/// use xrpl::{Client, xrp, types::{Asset, XChainBridge, builders::XChainCreateBridgeBuilder}};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = XChainCreateBridgeBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
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
pub type XChainCreateBridgeBuilder = TransactionBuilder<XChainCreateBridge>;

impl XChainCreateBridgeBuilder {
    /// Creates a new `XChainCreateBridgeBuilder` with the required signature reward and bridge config.
    pub fn new(
        account: impl AsRef<str>,
        signature_reward: impl Into<Amount>,
        xchain_bridge: impl Into<XChainBridge>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            XChainCreateBridge {
                signature_reward: signature_reward.into(),
                min_account_create_amount: None,
                xchain_bridge: xchain_bridge.into(),
            },
        )
    }

    /// Minimum XRP required to create an account on the issuing chain via this bridge.
    pub fn with_min_account_create_amount(
        mut self,
        amount: impl Into<Amount>,
    ) -> Self {
        self.transaction_type.min_account_create_amount = Some(amount.into());
        self
    }
}

impl TransactionTypeBuilder for XChainCreateBridge {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_amount(&self.signature_reward)?;
        if let Some(min_account_create_amount) = &self.min_account_create_amount
        {
            validate_amount(min_account_create_amount)?;
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::XChainCreateBridge(self))
    }
}
