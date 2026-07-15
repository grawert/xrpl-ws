use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    transactions::xchain::XChainClaim,
    validation::{validate_address, validate_amount},
    Amount, TransactionType, XChainBridge,
};

/// Builder for XRPL XChainClaim transactions.
///
/// Claims assets on the destination chain after a `XChainCommit` has been
/// attested by the witness servers.
///
/// # Example
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # use xrpl::types::Amount;
/// use xrpl::{Client, xrp, types::{Asset, XChainBridge, builders::XChainClaimBuilder}};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = XChainClaimBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     xrp!(100),
///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
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
pub type XChainClaimBuilder = TransactionBuilder<XChainClaim>;

impl XChainClaimBuilder {
    /// Creates a new `XChainClaimBuilder` with all required fields.
    pub fn new(
        account: impl AsRef<str>,
        amount: impl Into<Amount>,
        destination: impl AsRef<str>,
        xchain_bridge: impl Into<XChainBridge>,
        xchain_claim_id: impl AsRef<str>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            XChainClaim {
                amount: amount.into(),
                destination: destination.as_ref().to_string(),
                destination_tag: None,
                xchain_bridge: xchain_bridge.into(),
                xchain_claim_id: xchain_claim_id.as_ref().to_string(),
            },
        )
    }

    /// Sets the destination tag for routing within the recipient account.
    pub fn with_destination_tag(mut self, tag: u32) -> Self {
        self.transaction_type.destination_tag = Some(tag);
        self
    }
}

impl TransactionTypeBuilder for XChainClaim {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_amount(&self.amount)?;
        validate_address(&self.destination)?;
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::XChainClaim(self))
    }
}
