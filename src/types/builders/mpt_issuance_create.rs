use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    transactions::mpt::MPTokenIssuanceCreate, Amount, TransactionType,
};

/// Builder for XRPL MPTokenIssuanceCreate transactions.
///
/// Creates a new MPToken issuance on the ledger. The issuer account becomes
/// the controlling authority for the issuance.
///
/// Common flags — pass to [`with_flags`] using [`MPTokenIssuanceCreateFlags`]:
/// - [`CAN_LOCK`] — issuer can lock individual balances or the whole issuance
/// - [`REQUIRE_AUTH`] — holders must be authorized before holding
/// - [`CAN_ESCROW`] — tokens can be held in escrow
/// - [`CAN_TRADE`] — tokens can be traded on the DEX
/// - [`CAN_TRANSFER`] — holders can transfer tokens between accounts
/// - [`CAN_CLAWBACK`] — issuer can claw back tokens from holders
///
/// [`with_flags`]: TransactionBuilder::with_flags
/// [`MPTokenIssuanceCreateFlags`]: crate::types::MPTokenIssuanceCreateFlags
/// [`CAN_LOCK`]: crate::types::MPTokenIssuanceCreateFlags::CAN_LOCK
/// [`REQUIRE_AUTH`]: crate::types::MPTokenIssuanceCreateFlags::REQUIRE_AUTH
/// [`CAN_ESCROW`]: crate::types::MPTokenIssuanceCreateFlags::CAN_ESCROW
/// [`CAN_TRADE`]: crate::types::MPTokenIssuanceCreateFlags::CAN_TRADE
/// [`CAN_TRANSFER`]: crate::types::MPTokenIssuanceCreateFlags::CAN_TRANSFER
/// [`CAN_CLAWBACK`]: crate::types::MPTokenIssuanceCreateFlags::CAN_CLAWBACK
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::{MPTokenIssuanceCreateFlags, builders::MPTokenIssuanceCreateBuilder}};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = MPTokenIssuanceCreateBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
///     .with_asset_scale(2)
///     .with_transfer_fee(500)
///     .with_maximum_amount(1_000_000)
///     .with_flags(
///         MPTokenIssuanceCreateFlags::CAN_TRANSFER
///             | MPTokenIssuanceCreateFlags::CAN_LOCK,
///     )
///     .fill(&client)
///     .await?
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub type MPTokenIssuanceCreateBuilder =
    TransactionBuilder<MPTokenIssuanceCreate>;

impl MPTokenIssuanceCreateBuilder {
    /// Creates a new `MPTokenIssuanceCreateBuilder` with no optional fields pre-set.
    pub fn new(account: impl AsRef<str>) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            MPTokenIssuanceCreate {
                asset_scale: None,
                maximum_amount: None,
                mpt_metadata: None,
                transfer_fee: None,
            },
        )
    }

    /// Decimal precision of the token (number of digits after the decimal point).
    pub fn with_asset_scale(mut self, asset_scale: u8) -> Self {
        self.transaction_type.asset_scale = Some(asset_scale);
        self
    }

    /// Maximum number of tokens that can be distributed, e.g. `1_000_000`.
    pub fn with_maximum_amount(mut self, maximum_amount: u64) -> Self {
        self.transaction_type.maximum_amount = Some(maximum_amount.to_string());
        self
    }

    /// Hex-encoded metadata associated with the issuance.
    pub fn with_mpt_metadata(mut self, mpt_metadata: impl AsRef<str>) -> Self {
        self.transaction_type.mpt_metadata =
            Some(mpt_metadata.as_ref().to_string());
        self
    }

    /// Transfer fee in units of 1/100,000 of a percent (0–50000, i.e. 0%–50%).
    /// Requires [`MPTokenIssuanceCreateFlags::CAN_TRANSFER`] to also be set via [`with_flags`].
    ///
    /// [`MPTokenIssuanceCreateFlags::CAN_TRANSFER`]: crate::types::MPTokenIssuanceCreateFlags::CAN_TRANSFER
    /// [`with_flags`]: TransactionBuilder::with_flags
    pub fn with_transfer_fee(mut self, transfer_fee: u16) -> Self {
        self.transaction_type.transfer_fee = Some(transfer_fee);
        self
    }
}

impl TransactionTypeBuilder for MPTokenIssuanceCreate {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(fee) = self.transfer_fee
            && fee > 50_000
        {
            return Err(BuildError::InvalidTransferFee);
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::MPTokenIssuanceCreate(self))
    }
}
