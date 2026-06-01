use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    transactions::xchain::XChainAddAccountCreateAttestation, Amount,
    TransactionType, XChainBridge,
};

/// Builder for XRPL XChainAddAccountCreateAttestation transactions.
///
/// Submitted by a witness server to attest that a `XChainAccountCreateCommit`
/// occurred on the source chain.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # use xrpl::types::Amount;
/// use xrpl::{Client, xrp, types::{Asset, XChainBridge, builders::XChainAddAccountCreateAttestationBuilder}};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = XChainAddAccountCreateAttestationBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     xrp!(20),
///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "r3kmLJN5D28dHuH8vZNUZpMC4JP9X8RHsv",
///     "rN7n3473SaZBCG4dFL83w7PB5NMJhkMFKE",
///     "rGWrZyax5eXbi5gs49MRZKmskElsde6Rm1",
///     "ED5E6F48B2B1E8C7D2C3F5A4B6E8D9F0A1C2D3E4F5A6B7C8D9E0F1A2B3C4D5E6F",
///     "A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2",
///     xrp!(100),
///     0,
///     "1",
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
pub type XChainAddAccountCreateAttestationBuilder =
    TransactionBuilder<XChainAddAccountCreateAttestation>;

impl XChainAddAccountCreateAttestationBuilder {
    /// Creates a new `XChainAddAccountCreateAttestationBuilder` with all attestation fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        account: impl Into<String>,
        amount: impl Into<Amount>,
        attestation_reward_account: impl Into<String>,
        attestation_signer_account: impl Into<String>,
        destination: impl Into<String>,
        other_chain_source: impl Into<String>,
        public_key: impl Into<String>,
        signature: impl Into<String>,
        signature_reward: impl Into<Amount>,
        was_locking_chain_send: u8,
        xchain_account_create_count: impl Into<String>,
        xchain_bridge: impl Into<XChainBridge>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            XChainAddAccountCreateAttestation {
                amount: amount.into(),
                attestation_reward_account: attestation_reward_account.into(),
                attestation_signer_account: attestation_signer_account.into(),
                destination: destination.into(),
                other_chain_source: other_chain_source.into(),
                public_key: public_key.into(),
                signature: signature.into(),
                signature_reward: signature_reward.into(),
                was_locking_chain_send,
                xchain_account_create_count: xchain_account_create_count.into(),
                xchain_bridge: xchain_bridge.into(),
            },
        )
    }
}

impl TransactionTypeBuilder for XChainAddAccountCreateAttestation {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::XChainAddAccountCreateAttestation(self))
    }
}
