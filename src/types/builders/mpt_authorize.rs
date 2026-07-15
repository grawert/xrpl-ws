use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::{validate_address, validate_mpt_id},
    transactions::mpt::MPTokenAuthorize,
    Amount, TransactionType,
};

/// Builder for XRPL MPTokenAuthorize transactions.
///
/// Allows an account to hold an MPToken issuance, or allows an issuer to
/// authorize a specific holder when the issuance has
/// [`MPTokenIssuanceCreateFlags::REQUIRE_AUTH`] set.
/// To revoke authorization, pass [`MPTokenAuthorizeFlags::UNAUTHORIZE`] to
/// [`with_flags`].
///
/// [`MPTokenIssuanceCreateFlags::REQUIRE_AUTH`]: crate::types::MPTokenIssuanceCreateFlags::REQUIRE_AUTH
/// [`MPTokenAuthorizeFlags::UNAUTHORIZE`]: crate::types::MPTokenAuthorizeFlags::UNAUTHORIZE
/// [`with_flags`]: TransactionBuilder::with_flags
///
/// # Examples
///
/// Authorize a holder to receive an issuance:
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::MPTokenAuthorizeBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = MPTokenAuthorizeBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "0000000024B5A7AE55A3019B1C7B38FBA04BEF0CEF2D6F48",
/// )
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
///
/// Holder revoking their own authorization:
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::{MPTokenAuthorizeFlags, builders::MPTokenAuthorizeBuilder}};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = MPTokenAuthorizeBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "0000000024B5A7AE55A3019B1C7B38FBA04BEF0CEF2D6F48",
/// )
/// .with_flags(MPTokenAuthorizeFlags::UNAUTHORIZE)
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
///
/// Issuer revoking a specific holder's authorization:
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::{MPTokenAuthorizeFlags, builders::MPTokenAuthorizeBuilder}};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = MPTokenAuthorizeBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "0000000024B5A7AE55A3019B1C7B38FBA04BEF0CEF2D6F48",
/// )
/// .with_holder("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")
/// .with_flags(MPTokenAuthorizeFlags::UNAUTHORIZE)
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type MPTokenAuthorizeBuilder = TransactionBuilder<MPTokenAuthorize>;

impl MPTokenAuthorizeBuilder {
    /// Creates a new `MPTokenAuthorizeBuilder` for the given issuance ID.
    pub fn new(
        account: impl AsRef<str>,
        mpt_issuance_id: impl AsRef<str>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            MPTokenAuthorize {
                mpt_issuance_id: mpt_issuance_id.as_ref().to_string(),
                holder: None,
            },
        )
    }

    /// Authorize a specific holder (issuer-side authorization).
    pub fn with_holder(mut self, holder: impl AsRef<str>) -> Self {
        self.transaction_type.holder = Some(holder.as_ref().to_string());
        self
    }
}

impl TransactionTypeBuilder for MPTokenAuthorize {
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
        Ok(TransactionType::MPTokenAuthorize(self))
    }
}
