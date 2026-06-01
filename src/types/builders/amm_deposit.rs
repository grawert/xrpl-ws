use crate::types::{
    Amount, Asset,
    builders::{BuildError, TransactionBuilder, TransactionTypeBuilder},
    transactions::{TransactionType, amm::AMMDeposit},
    validation::{validate_amount, ValidationError},
};

/// Builder for XRPL AMM deposit transactions.
///
/// # Example
/// ```rust
/// use xrpl::types::{Amount, Asset, builders::AMMDepositBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let asset = Asset::xrp();
///     let asset2 = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
///     let amm_deposit = AMMDepositBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", asset, asset2)
///         .with_amount(Amount::drops("10000000")?)
///         .with_amount2(Amount::issued_currency("100", "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?)
///         .build()?;
///     Ok(())
/// }
/// ```
pub type AMMDepositBuilder = TransactionBuilder<AMMDeposit>;

impl AMMDepositBuilder {
    /// Create a new AMM deposit builder
    pub fn new(
        account: impl Into<String>,
        asset: impl Into<Asset>,
        asset2: impl Into<Asset>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            AMMDeposit {
                asset: asset.into(),
                asset2: asset2.into(),
                amount: None,
                amount2: None,
                e_price: None,
                lp_token_out: None,
                trading_fee: None,
            },
        )
    }

    /// Sets the maximum amount of the first asset to deposit.
    pub fn with_amount(mut self, amount: impl Into<Amount>) -> Self {
        self.transaction_type.amount = Some(amount.into());
        self
    }

    /// Sets the maximum amount of the second asset to deposit.
    pub fn with_amount2(mut self, amount2: impl Into<Amount>) -> Self {
        self.transaction_type.amount2 = Some(amount2.into());
        self
    }

    /// Sets the effective price limit per LP token.
    pub fn with_e_price(mut self, e_price: impl Into<Amount>) -> Self {
        self.transaction_type.e_price = Some(e_price.into());
        self
    }

    /// Sets the exact number of LP tokens the depositor wants to receive.
    pub fn with_lp_token_out(
        mut self,
        lp_token_out: impl Into<Amount>,
    ) -> Self {
        self.transaction_type.lp_token_out = Some(lp_token_out.into());
        self
    }

    /// Sets the trading fee vote (0–1000, where 1000 = 1%).
    pub fn with_trading_fee(mut self, trading_fee: u16) -> Self {
        self.transaction_type.trading_fee = Some(trading_fee);
        self
    }
}

impl TransactionTypeBuilder for AMMDeposit {
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
        if let Some(ref lp_token_out) = self.lp_token_out {
            validate_amount(lp_token_out)?;
        }
        if let Some(fee) = self.trading_fee
            && fee > 1000
        {
            return Err(ValidationError::InvalidAmount(
                "Trading fee cannot exceed 1000 (1%)".into(),
            )
            .into());
        }
        let has_amount = self.amount.is_some();
        let has_amount2 = self.amount2.is_some();
        let has_e_price = self.e_price.is_some();
        let has_lp_token_out = self.lp_token_out.is_some();

        if !has_amount && !has_amount2 && !has_e_price && !has_lp_token_out {
            return Err(ValidationError::InvalidAmount(
                "At least one of Amount, Amount2, EPrice, or LPTokenOut must be set".into(),
            )
            .into());
        }
        if has_amount2 && !has_amount {
            return Err(ValidationError::InvalidAmount(
                "Amount2 requires Amount".into(),
            )
            .into());
        }
        if has_e_price && !has_amount {
            return Err(ValidationError::InvalidAmount(
                "EPrice requires Amount".into(),
            )
            .into());
        }
        if has_e_price && has_lp_token_out {
            return Err(ValidationError::InvalidAmount(
                "EPrice and LPTokenOut are mutually exclusive".into(),
            )
            .into());
        }
        if has_amount2 && has_lp_token_out {
            return Err(ValidationError::InvalidAmount(
                "Amount2 and LPTokenOut are mutually exclusive".into(),
            )
            .into());
        }
        if has_amount2 && has_e_price {
            return Err(ValidationError::InvalidAmount(
                "Amount2 and EPrice are mutually exclusive".into(),
            )
            .into());
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::AMMDeposit(self))
    }
}
