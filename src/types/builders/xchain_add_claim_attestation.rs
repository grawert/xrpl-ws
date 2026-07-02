use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    transactions::xchain::XChainAddClaimAttestation, Amount, TransactionType,
    XChainBridge,
};

/// Builder for XRPL XChainAddClaimAttestation transactions.
///
/// Submitted by a witness server to attest that a `XChainCommit` occurred on
/// the source chain. Collects attestations until the quorum is met, then
/// releases the funds to the destination.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # use xrpl::types::Amount;
/// use xrpl::{Client, xrp, types::{Asset, XChainBridge, builders::XChainAddClaimAttestationBuilder}};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = XChainAddClaimAttestationBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     xrp!(100),
///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "r3kmLJN5D28dHuH8vZNUZpMC4JP9X8RHsv",
///     "ED5E6F48B2B1E8C7D2C3F5A4B6E8D9F0A1C2D3E4F5A6B7C8D9E0F1A2B3C4D5E6F",
///     "A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2",
///     0,
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
pub type XChainAddClaimAttestationBuilder =
    TransactionBuilder<XChainAddClaimAttestation>;

impl XChainAddClaimAttestationBuilder {
    /// Creates a new `XChainAddClaimAttestationBuilder` with the required attestation fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        account: impl AsRef<str>,
        amount: impl Into<Amount>,
        attestation_reward_account: impl AsRef<str>,
        attestation_signer_account: impl AsRef<str>,
        public_key: impl AsRef<str>,
        signature: impl AsRef<str>,
        was_locking_chain_send: u8,
        xchain_bridge: impl Into<XChainBridge>,
        xchain_claim_id: impl AsRef<str>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            XChainAddClaimAttestation {
                amount: amount.into(),
                attestation_reward_account: attestation_reward_account
                    .as_ref()
                    .to_string(),
                attestation_signer_account: attestation_signer_account
                    .as_ref()
                    .to_string(),
                destination: None,
                other_chain_source: String::new(),
                public_key: public_key.as_ref().to_string(),
                signature: signature.as_ref().to_string(),
                was_locking_chain_send,
                xchain_bridge: xchain_bridge.into(),
                xchain_claim_id: xchain_claim_id.as_ref().to_string(),
            },
        )
    }

    /// Source account on the other chain that initiated the `XChainCommit`.
    pub fn with_other_chain_source(mut self, source: impl AsRef<str>) -> Self {
        self.transaction_type.other_chain_source = source.as_ref().to_string();
        self
    }

    /// Destination account on this chain to receive the funds.
    pub fn with_destination(mut self, destination: impl AsRef<str>) -> Self {
        self.transaction_type.destination =
            Some(destination.as_ref().to_string());
        self
    }
}

impl TransactionTypeBuilder for XChainAddClaimAttestation {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::XChainAddClaimAttestation(self))
    }
}
