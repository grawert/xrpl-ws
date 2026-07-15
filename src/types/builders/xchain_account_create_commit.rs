use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    transactions::xchain::XChainAccountCreateCommit,
    validation::{validate_address, validate_amount},
    Amount, TransactionType, XChainBridge,
};

/// Builder for XRPL XChainAccountCreateCommit transactions.
///
/// Creates an account on the issuing chain via the bridge. This is used when
/// the destination account does not yet exist on the issuing chain.
///
/// # Example
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # use xrpl::types::Amount;
/// use xrpl::{Client, xrp, types::{Asset, XChainBridge, builders::XChainAccountCreateCommitBuilder}};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = XChainAccountCreateCommitBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     xrp!(20),
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
pub type XChainAccountCreateCommitBuilder =
    TransactionBuilder<XChainAccountCreateCommit>;

impl XChainAccountCreateCommitBuilder {
    /// Creates a new `XChainAccountCreateCommitBuilder` with all required fields.
    pub fn new(
        account: impl AsRef<str>,
        amount: impl Into<Amount>,
        destination: impl AsRef<str>,
        signature_reward: impl Into<Amount>,
        xchain_bridge: impl Into<XChainBridge>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            XChainAccountCreateCommit {
                amount: amount.into(),
                destination: destination.as_ref().to_string(),
                signature_reward: signature_reward.into(),
                xchain_bridge: xchain_bridge.into(),
            },
        )
    }
}

impl TransactionTypeBuilder for XChainAccountCreateCommit {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_amount(&self.amount)?;
        validate_amount(&self.signature_reward)?;
        validate_address(&self.destination)?;
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::XChainAccountCreateCommit(self))
    }
}
