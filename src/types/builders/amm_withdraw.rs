use crate::types::{
    Amount, Asset,
    builders::{BuildError, TransactionBuilder, TransactionTypeBuilder},
    transactions::{TransactionType, amm::AMMWithdraw},
    validation::{validate_amount, ValidationError},
};

/// Builder for XRPL AMM withdraw transactions.
///
/// # Example
/// ```rust
/// use xrpl::types::{Amount, Asset, builders::AMMWithdrawBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let asset = Asset::xrp();
///     let asset2 = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
///     let amm_withdraw = AMMWithdrawBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", asset, asset2)
///         .with_lp_token_in(Amount::issued_currency("100", "03930D02208264E2E40EC1B0C09E4DB96EE197B1", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?)
///         .build()?;
///     Ok(())
/// }
/// ```
pub type AMMWithdrawBuilder = TransactionBuilder<AMMWithdraw>;

impl AMMWithdrawBuilder {
    /// Creates a new AMM withdraw builder
    pub fn new(
        account: impl AsRef<str>,
        asset: impl Into<Asset>,
        asset2: impl Into<Asset>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            AMMWithdraw {
                asset: asset.into(),
                asset2: asset2.into(),
                amount: None,
                amount2: None,
                e_price: None,
                lp_token_in: None,
            },
        )
    }

    /// Sets the minimum amount of the first asset to receive.
    pub fn with_amount(mut self, amount: impl Into<Amount>) -> Self {
        self.transaction_type.amount = Some(amount.into());
        self
    }

    /// Sets the minimum amount of the second asset to receive.
    pub fn with_amount2(mut self, amount2: impl Into<Amount>) -> Self {
        self.transaction_type.amount2 = Some(amount2.into());
        self
    }

    /// Sets the effective price limit per LP token.
    pub fn with_e_price(mut self, e_price: impl Into<Amount>) -> Self {
        self.transaction_type.e_price = Some(e_price.into());
        self
    }

    /// Sets the exact number of LP tokens to redeem.
    pub fn with_lp_token_in(mut self, lp_token_in: impl Into<Amount>) -> Self {
        self.transaction_type.lp_token_in = Some(lp_token_in.into());
        self
    }
}

impl TransactionTypeBuilder for AMMWithdraw {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(ref amount) = self.amount {
            validate_amount(amount)?;
        }
        if let Some(ref amount2) = self.amount2 {
            validate_amount(amount2)?;
        }
        if let Some(ref e_price) = self.e_price {
            validate_amount(e_price)?;
        }
        if let Some(ref lp_token_in) = self.lp_token_in {
            validate_amount(lp_token_in)?;
        }
        let has_amount = self.amount.is_some();
        let has_amount2 = self.amount2.is_some();
        let has_e_price = self.e_price.is_some();
        let has_lp_token_in = self.lp_token_in.is_some();

        if has_amount2 && !has_amount {
            return Err(ValidationError::InvalidAmount(
                "Amount2 requires Amount".into(),
            )
            .into());
        }
        if has_e_price && !has_lp_token_in {
            return Err(ValidationError::InvalidAmount(
                "EPrice requires LPTokenIn".into(),
            )
            .into());
        }
        if has_amount2 && has_lp_token_in {
            return Err(ValidationError::InvalidAmount(
                "Amount2 and LPTokenIn are mutually exclusive".into(),
            )
            .into());
        }
        if has_amount && has_e_price && !has_lp_token_in {
            return Err(ValidationError::InvalidAmount(
                "EPrice requires LPTokenIn".into(),
            )
            .into());
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::AMMWithdraw(self))
    }
}
