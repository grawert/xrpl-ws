use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::validate_mpt_id, transactions::mpt::MPTokenIssuanceDestroy,
    Amount, TransactionType,
};

/// Builder for XRPL MPTokenIssuanceDestroy transactions.
///
/// Removes an MPToken issuance from the ledger. The issuance must have an
/// outstanding amount of zero before it can be destroyed.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::MPTokenIssuanceDestroyBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = MPTokenIssuanceDestroyBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "0000000024B5A7AE55A3019B1C7B38FBA04BEF0CEF2D6F48",
/// )
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type MPTokenIssuanceDestroyBuilder =
    TransactionBuilder<MPTokenIssuanceDestroy>;

impl MPTokenIssuanceDestroyBuilder {
    /// Creates a new `MPTokenIssuanceDestroyBuilder` for the given issuance ID.
    pub fn new(
        account: impl AsRef<str>,
        mpt_issuance_id: impl AsRef<str>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            MPTokenIssuanceDestroy {
                mpt_issuance_id: mpt_issuance_id.as_ref().to_string(),
            },
        )
    }
}

impl TransactionTypeBuilder for MPTokenIssuanceDestroy {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_mpt_id(&self.mpt_issuance_id)?;
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::MPTokenIssuanceDestroy(self))
    }
}
