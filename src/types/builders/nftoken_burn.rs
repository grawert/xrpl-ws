use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::validate_address, transactions::nft::NFTokenBurn, Amount,
    TransactionType,
};

/// Builder for XRPL NFTokenBurn transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::NFTokenBurnBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = NFTokenBurnBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65",
/// )
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type NFTokenBurnBuilder = TransactionBuilder<NFTokenBurn>;

impl NFTokenBurnBuilder {
    /// Creates a new `NFTokenBurnBuilder` targeting the specified token ID.
    pub fn new(
        account: impl Into<String>,
        nftoken_id: impl Into<String>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            NFTokenBurn { nftoken_id: nftoken_id.into(), owner: None },
        )
    }

    /// Sets the current owner; required when the issuer (not the owner) submits the burn.
    pub fn with_owner(mut self, owner: impl Into<String>) -> Self {
        self.transaction_type.owner = Some(owner.into());
        self
    }
}

impl TransactionTypeBuilder for NFTokenBurn {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(owner) = &self.owner {
            validate_address(owner)?;
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::NFTokenBurn(self))
    }
}
