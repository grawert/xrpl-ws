use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    transactions::xchain::XChainAddAccountCreateAttestation,
    validation::{validate_address, validate_amount},
    Amount, TransactionType,
};

/// Builder for XRPL XChainAddAccountCreateAttestation transactions.
///
/// Submitted by a witness server to attest that a `XChainAccountCreateCommit`
/// occurred on the source chain.
///
/// # Example
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # use xrpl::types::Amount;
/// use xrpl::{Client, xrp, types::{Asset, XChainAddAccountCreateAttestation, XChainBridge, builders::XChainAddAccountCreateAttestationBuilder}};
/// let client = Client::new("wss://xrplcluster.com");
/// let attestation = XChainAddAccountCreateAttestation {
///     amount: xrp!(20),
///     attestation_reward_account: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".into(),
///     attestation_signer_account: "r3kmLJN5D28dHuH8vZNUZpMC4JP9X8RHsv".into(),
///     destination: "rN7n3473SaZBCG4dFL83w7PB5NMJhkMFKE".into(),
///     other_chain_source: "rGWrZyax5eXbi5gs49MRZKmskElsde6Rm1".into(),
///     public_key: "ED5E6F48B2B1E8C7D2C3F5A4B6E8D9F0A1C2D3E4F5A6B7C8D9E0F1A2B3C4D5E6F".into(),
///     signature: "A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2".into(),
///     signature_reward: xrp!(100),
///     was_locking_chain_send: 0,
///     xchain_account_create_count: "1".into(),
///     xchain_bridge: XChainBridge {
///         locking_chain_door: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         locking_chain_issue: Asset::xrp(),
///         issuing_chain_door: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
///         issuing_chain_issue: Asset::xrp(),
///     },
/// };
/// let tx = XChainAddAccountCreateAttestationBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     attestation,
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
    /// Creates a new `XChainAddAccountCreateAttestationBuilder` from an
    /// [`XChainAddAccountCreateAttestation`] struct.
    pub fn new(
        account: impl AsRef<str>,
        attestation: XChainAddAccountCreateAttestation,
    ) -> Self {
        Self::init(account, 0, Amount::default(), attestation)
    }
}

impl TransactionTypeBuilder for XChainAddAccountCreateAttestation {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_amount(&self.amount)?;
        validate_amount(&self.signature_reward)?;
        validate_address(&self.attestation_reward_account)?;
        validate_address(&self.attestation_signer_account)?;
        validate_address(&self.destination)?;
        validate_address(&self.other_chain_source)?;
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::XChainAddAccountCreateAttestation(self))
    }
}
