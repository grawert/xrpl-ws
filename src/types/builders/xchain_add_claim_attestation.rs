use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    transactions::xchain::XChainAddClaimAttestation,
    validation::{validate_address, validate_amount},
    Amount, TransactionType,
};

/// Builder for XRPL XChainAddClaimAttestation transactions.
///
/// Submitted by a witness server to attest that a `XChainCommit` occurred on
/// the source chain. Collects attestations until the quorum is met, then
/// releases the funds to the destination.
///
/// # Example
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # use xrpl::types::Amount;
/// use xrpl::{Client, xrp, types::{Asset, XChainAddClaimAttestation, XChainBridge, builders::XChainAddClaimAttestationBuilder}};
/// let client = Client::new("wss://xrplcluster.com");
/// let attestation = XChainAddClaimAttestation {
///     amount: xrp!(100),
///     attestation_reward_account: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".into(),
///     attestation_signer_account: "r3kmLJN5D28dHuH8vZNUZpMC4JP9X8RHsv".into(),
///     destination: None,
///     other_chain_source: String::new(),
///     public_key: "ED5E6F48B2B1E8C7D2C3F5A4B6E8D9F0A1C2D3E4F5A6B7C8D9E0F1A2B3C4D5E6F".into(),
///     signature: "A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2".into(),
///     was_locking_chain_send: 0,
///     xchain_bridge: XChainBridge {
///         locking_chain_door: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         locking_chain_issue: Asset::xrp(),
///         issuing_chain_door: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
///         issuing_chain_issue: Asset::xrp(),
///     },
///     xchain_claim_id: "1".into(),
/// };
/// let tx = XChainAddClaimAttestationBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     attestation,
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
    /// Creates a new `XChainAddClaimAttestationBuilder` from an [`XChainAddClaimAttestation`]
    /// struct. Call [`with_other_chain_source`](Self::with_other_chain_source) to set the
    /// other chain source (required) and [`with_destination`](Self::with_destination) to
    /// set the optional destination field.
    pub fn new(
        account: impl AsRef<str>,
        attestation: XChainAddClaimAttestation,
    ) -> Self {
        Self::init(account, 0, Amount::default(), attestation)
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
        validate_amount(&self.amount)?;
        validate_address(&self.attestation_reward_account)?;
        validate_address(&self.attestation_signer_account)?;
        if let Some(dest) = &self.destination {
            validate_address(dest)?;
        }
        if !self.other_chain_source.is_empty() {
            validate_address(&self.other_chain_source)?;
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::XChainAddClaimAttestation(self))
    }
}
