use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    transactions::xchain::XChainCommit,
    validation::{validate_address, validate_amount},
    Amount, TransactionType, XChainBridge,
};

/// Builder for XRPL XChainCommit transactions.
///
/// Locks or burns assets on the source chain as part of a cross-chain transfer.
/// A `XChainCreateClaimID` must be submitted on the destination chain first to
/// obtain the `xchain_claim_id` value.
///
/// # Example
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # use xrpl::types::Amount;
/// use xrpl::{Client, xrp, types::{Asset, XChainBridge, builders::XChainCommitBuilder}};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = XChainCommitBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     xrp!(100),
///     XChainBridge {
///         locking_chain_door: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         locking_chain_issue: Asset::xrp(),
///         issuing_chain_door: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
///         issuing_chain_issue: Asset::xrp(),
///     },
///     "1",
/// )
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type XChainCommitBuilder = TransactionBuilder<XChainCommit>;

impl XChainCommitBuilder {
    /// Creates a new `XChainCommitBuilder` with all required fields.
    pub fn new(
        account: impl AsRef<str>,
        amount: impl Into<Amount>,
        xchain_bridge: impl Into<XChainBridge>,
        xchain_claim_id: impl AsRef<str>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            XChainCommit {
                amount: amount.into(),
                other_chain_destination: None,
                xchain_bridge: xchain_bridge.into(),
                xchain_claim_id: xchain_claim_id.as_ref().to_string(),
            },
        )
    }

    /// Destination account on the other chain. If omitted, the claiming account
    /// specified in `XChainClaim` is used.
    pub fn with_other_chain_destination(
        mut self,
        destination: impl AsRef<str>,
    ) -> Self {
        self.transaction_type.other_chain_destination =
            Some(destination.as_ref().to_string());
        self
    }
}

impl TransactionTypeBuilder for XChainCommit {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_amount(&self.amount)?;
        if let Some(other) = &self.other_chain_destination {
            validate_address(other)?;
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::XChainCommit(self))
    }
}
