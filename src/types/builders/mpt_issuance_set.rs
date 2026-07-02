use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::{validate_address, validate_mpt_id},
    transactions::mpt::MPTokenIssuanceSet,
    Amount, TransactionType,
};

/// Builder for XRPL MPTokenIssuanceSet transactions.
///
/// Locks or unlocks an MPToken issuance or a specific holder's balance.
/// Requires the issuance to have been created with
/// [`MPTokenIssuanceCreateFlags::CAN_LOCK`].
///
/// [`MPTokenIssuanceCreateFlags::CAN_LOCK`]: crate::types::MPTokenIssuanceCreateFlags::CAN_LOCK
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::{MPTokenIssuanceSetAction, builders::MPTokenIssuanceSetBuilder}};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = MPTokenIssuanceSetBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "0000000024B5A7AE55A3019B1C7B38FBA04BEF0CEF2D6F48",
/// )
/// .with_flags(MPTokenIssuanceSetAction::Lock)
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type MPTokenIssuanceSetBuilder = TransactionBuilder<MPTokenIssuanceSet>;

impl MPTokenIssuanceSetBuilder {
    /// Creates a new `MPTokenIssuanceSetBuilder` for the given issuance ID.
    pub fn new(
        account: impl AsRef<str>,
        mpt_issuance_id: impl AsRef<str>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            MPTokenIssuanceSet {
                mpt_issuance_id: mpt_issuance_id.as_ref().to_string(),
                holder: None,
            },
        )
    }

    /// Lock or unlock the balance of a specific holder instead of the whole issuance.
    pub fn with_holder(mut self, holder: impl AsRef<str>) -> Self {
        self.transaction_type.holder = Some(holder.as_ref().to_string());
        self
    }
}

impl TransactionTypeBuilder for MPTokenIssuanceSet {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_mpt_id(&self.mpt_issuance_id)?;
        if let Some(holder) = &self.holder {
            validate_address(holder)?;
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::MPTokenIssuanceSet(self))
    }
}
