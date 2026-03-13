use crate::types::{
    Amount, AuthAccountWrapper, ValidationError, validate_amount,
    validate_address,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Builder for XRPL AMM bid transactions.
///
/// # Example
/// ```no_run
/// use xrpl::types::{Amount, builders::AMMBidBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let asset = Amount::drops("100000000")?;
///     let asset2 = Amount::issued_currency("1000", "USD", "rIssuer")?;
///     let amm_bid = AMMBidBuilder::new(asset, asset2)
///         .bid_min(Amount::issued_currency("10", "USD", "rIssuer")?)
///         .bid_max(Amount::issued_currency("50", "USD", "rIssuer")?)
///         .build()?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AMMBidBuilder {
    asset: Amount,
    asset2: Amount,
    bid_min: Option<Amount>,
    bid_max: Option<Amount>,
    auth_accounts: Option<Vec<AuthAccountWrapper>>,
}

impl AMMBidBuilder {
    /// Create a new AMM bid builder
    pub fn new(asset: Amount, asset2: Amount) -> Self {
        Self {
            asset,
            asset2,
            bid_min: None,
            bid_max: None,
            auth_accounts: None,
        }
    }

    /// Set minimum bid amount
    pub fn bid_min(mut self, amount: Amount) -> Self {
        self.bid_min = Some(amount);
        self
    }

    /// Set maximum bid amount
    pub fn bid_max(mut self, amount: Amount) -> Self {
        self.bid_max = Some(amount);
        self
    }

    /// Set bid range (min and max)
    pub fn bid_range(mut self, min: Amount, max: Amount) -> Self {
        self.bid_min = Some(min);
        self.bid_max = Some(max);
        self
    }

    /// Add authorized accounts (up to 4)
    pub fn with_auth_accounts(
        mut self,
        accounts: Vec<String>,
    ) -> Result<Self, ValidationError> {
        if accounts.len() > 4 {
            return Err(ValidationError::InvalidAddress(
                "Cannot authorize more than 4 accounts".into(),
            ));
        }

        // Validate all addresses and check for duplicates
        let mut validated_accounts = Vec::new();
        for account in accounts {
            validate_address(&account)?;
            if validated_accounts
                .iter()
                .any(|a: &AuthAccountWrapper| a.auth_account.account == account)
            {
                return Err(ValidationError::InvalidAddress(
                    "Duplicate accounts not allowed".into(),
                ));
            }
            validated_accounts.push(AuthAccountWrapper {
                auth_account: crate::types::AuthAccount { account },
            });
        }

        self.auth_accounts = Some(validated_accounts);
        Ok(self)
    }

    /// Build the bid transaction fields
    pub fn build(self) -> Result<AMMBidFields, ValidationError> {
        validate_amount(&self.asset)?;
        validate_amount(&self.asset2)?;

        if let Some(ref bid_min) = self.bid_min {
            validate_amount(bid_min)?;
        }
        if let Some(ref bid_max) = self.bid_max {
            validate_amount(bid_max)?;
        }

        Ok(AMMBidFields {
            asset: self.asset,
            asset2: self.asset2,
            bid_min: self.bid_min,
            bid_max: self.bid_max,
            auth_accounts: self.auth_accounts,
        })
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AMMBidFields {
    #[serde(rename = "Asset")]
    pub asset: Amount,
    #[serde(rename = "Asset2")]
    pub asset2: Amount,
    #[serde(rename = "BidMin")]
    pub bid_min: Option<Amount>,
    #[serde(rename = "BidMax")]
    pub bid_max: Option<Amount>,
    #[serde(rename = "AuthAccounts")]
    pub auth_accounts: Option<Vec<AuthAccountWrapper>>,
}
