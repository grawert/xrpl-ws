use crate::types::{
    Amount, Asset, AuthAccountWrapper,
    builders::{BuildError, TransactionBuilder, TransactionTypeBuilder},
    transactions::{TransactionType, amm::AMMBid},
    validation::{validate_address, validate_amount, ValidationError},
};

/// Builder for XRPL AMM bid transactions.
///
/// # Example
/// ```rust
/// use xrpl::types::{Asset, Amount, builders::AMMBidBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let asset = Asset::xrp();
///     let asset2 = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
///     let amm_bid = AMMBidBuilder::new("rPT0Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", asset, asset2)
///         .with_bid_min(Amount::issued_currency("10", "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?)
///         .with_bid_max(Amount::issued_currency("50", "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?)
///         .build()?;
///     Ok(())
/// }
/// ```
pub type AMMBidBuilder = TransactionBuilder<AMMBid>;

impl AMMBidBuilder {
    /// Create a new AMM bid builder
    pub fn new(
        account: impl AsRef<str>,
        asset: impl Into<Asset>,
        asset2: impl Into<Asset>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            AMMBid {
                asset: asset.into(),
                asset2: asset2.into(),
                bid_min: None,
                bid_max: None,
                auth_accounts: None,
            },
        )
    }

    /// Sets the minimum LP token bid amount for the auction slot.
    pub fn with_bid_min(mut self, amount: impl Into<Amount>) -> Self {
        self.transaction_type.bid_min = Some(amount.into());
        self
    }

    /// Sets the maximum LP token bid amount for the auction slot.
    pub fn with_bid_max(mut self, amount: impl Into<Amount>) -> Self {
        self.transaction_type.bid_max = Some(amount.into());
        self
    }

    /// Sets both the minimum and maximum LP token bid amounts.
    pub fn with_bid_range(
        mut self,
        min: impl Into<Amount>,
        max: impl Into<Amount>,
    ) -> Self {
        self.transaction_type.bid_min = Some(min.into());
        self.transaction_type.bid_max = Some(max.into());
        self
    }

    /// Add authorized accounts (up to 4)
    pub fn with_auth_accounts(
        mut self,
        accounts: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Self {
        let accounts = accounts
            .into_iter()
            .map(|s| AuthAccountWrapper {
                auth_account: crate::types::AuthAccount {
                    account: s.as_ref().to_string(),
                },
            })
            .collect();
        self.transaction_type.auth_accounts = Some(accounts);
        self
    }
}

impl TransactionTypeBuilder for AMMBid {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(ref bid_min) = self.bid_min {
            validate_amount(bid_min)?;
        }
        if let Some(ref bid_max) = self.bid_max {
            validate_amount(bid_max)?;
        }

        if let Some(ref auth_accounts) = self.auth_accounts {
            if auth_accounts.len() > 4 {
                return Err(ValidationError::InvalidAddress(
                    "Cannot authorize more than 4 accounts".into(),
                )
                .into());
            }

            let mut validated_accounts = Vec::new();
            for account_wrapper in auth_accounts {
                let account = &account_wrapper.auth_account.account;
                validate_address(account)?;
                if validated_accounts.contains(account) {
                    return Err(ValidationError::InvalidAddress(
                        "Duplicate accounts not allowed".into(),
                    )
                    .into());
                }
                validated_accounts.push(account.clone());
            }
        }

        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::AMMBid(self))
    }
}
