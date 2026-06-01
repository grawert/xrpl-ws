use std::fmt;
use serde::{Deserialize, Serialize};
use super::Amount;
use super::validation::{
    ValidationError, validate_address, validate_currency_code, validate_mpt_id,
    validate_amount_string,
};

/// Identifies a tradable asset on the XRPL without specifying an amount.
///
/// Use an `Asset` to describe a pool side, order book entry, or trust-line
/// target where only the asset identity matters, not a concrete quantity.
///
/// # Construction
///
/// 1. **Constructors** — [`Asset::xrp`] is infallible; [`Asset::token`] and
///    [`Asset::mpt`] validate their inputs and return a [`Result`].
/// 2. **From an [`Amount`]** — drop the value with
///    [`Asset::try_from(amount)`][`TryFrom<Amount>`] or
///    [`Asset::try_from(&amount)`][`TryFrom<&Amount>`].
/// 3. **Your own domain type** — implement [`From<MyType> for Asset`] once and
///    pass `MyType` directly to any builder method (they all accept
///    `impl Into<Asset>`).
///
/// To go the other direction — pair an `Asset` with a value to produce an
/// [`Amount`] — use [`Asset::amount_with`].
///
/// [`TryFrom<Amount>`]: Asset#impl-TryFrom<Amount>-for-Asset
/// [`TryFrom<&Amount>`]: Asset#impl-TryFrom<%26Amount>-for-Asset
/// [`From<MyType> for Asset`]: From
///
/// # Examples
///
/// Constructors:
/// ```rust
/// use xrpl::types::Asset;
///
/// let xrp   = Asset::xrp();
/// let usd   = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap();
/// let mpt   = Asset::mpt("0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47").unwrap();
/// ```
///
/// Round-trip with [`Amount`]:
/// ```rust
/// use xrpl::types::{Amount, Asset};
///
/// let asset = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap();
/// let amount = asset.amount_with("100").unwrap();
/// let stripped = Asset::try_from(&amount).unwrap();
/// assert_eq!(stripped, asset);
/// ```
///
/// Interop with your own domain type:
/// ```rust
/// use xrpl::types::Asset;
///
/// enum Currency { Native, Usd, Eur }
///
/// const USD_ISSUER: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
/// const EUR_ISSUER: &str = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";
///
/// impl From<Currency> for Asset {
///     fn from(c: Currency) -> Self {
///         match c {
///             Currency::Native => Asset::xrp(),
///             Currency::Usd => Asset::token("USD", USD_ISSUER).unwrap(),
///             Currency::Eur => Asset::token("EUR", EUR_ISSUER).unwrap(),
///         }
///     }
/// }
///
/// // Now `Currency::Usd` can be passed wherever `impl Into<Asset>` is expected,
/// // including AMM builder methods.
/// let asset: Asset = Currency::Usd.into();
/// ```
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Asset {
    /// Issued-currency token identified by currency code and issuer address.
    Token {
        /// 3-character standard currency code or 40-hex non-standard code.
        currency: String,
        /// r-address of the token issuer.
        issuer: String,
    },
    /// Multi-Purpose Token identified by its 48-character hex issuance ID.
    Mpt {
        /// 48-character hex MPT issuance ID.
        mpt_issuance_id: String,
    },
    /// Native XRP asset (serialized with `"currency": "XRP"`).
    Xrp {
        /// Always `"XRP"`.
        currency: String,
    },
}

impl Asset {
    /// Returns an XRP asset descriptor.
    pub fn xrp() -> Self {
        Asset::Xrp { currency: "XRP".to_string() }
    }

    /// Returns an issued-currency token asset descriptor, validating the currency code and issuer address.
    pub fn token<C, I>(currency: C, issuer: I) -> Result<Self, ValidationError>
    where
        C: Into<String>,
        I: Into<String>,
    {
        let currency = currency.into();
        let issuer = issuer.into();
        validate_currency_code(&currency, false)?;
        validate_address(&issuer)?;

        Ok(Asset::Token { currency, issuer })
    }

    /// Returns an MPT asset descriptor, validating the 48-character hex issuance ID.
    pub fn mpt<I>(mpt_issuance_id: I) -> Result<Self, ValidationError>
    where
        I: Into<String>,
    {
        let mpt_issuance_id = mpt_issuance_id.into();
        validate_mpt_id(&mpt_issuance_id)?;

        Ok(Asset::Mpt { mpt_issuance_id })
    }

    /// Returns the currency code. Returns `"XRP"` for XRP assets and an empty string for MPTs.
    ///
    /// # Example
    /// ```rust
    /// use xrpl::types::Asset;
    /// assert_eq!(Asset::xrp().currency(), "XRP");
    /// assert_eq!(Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap().currency(), "USD");
    /// ```
    pub fn currency(&self) -> &str {
        match self {
            Asset::Xrp { currency } => currency,
            Asset::Token { currency, .. } => currency,
            Asset::Mpt { .. } => "",
        }
    }

    /// Returns the issuer address if this is a token asset, otherwise `None`.
    ///
    /// # Example
    /// ```rust
    /// use xrpl::types::Asset;
    /// let usd = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap();
    /// assert_eq!(usd.issuer(), Some("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"));
    /// ```
    pub fn issuer(&self) -> Option<&str> {
        match self {
            Asset::Token { issuer, .. } => Some(issuer),
            _ => None,
        }
    }

    /// Returns the MPT issuance ID if this is an MPT asset, otherwise `None`.
    ///
    /// # Example
    /// ```rust
    /// use xrpl::types::Asset;
    /// let mpt = Asset::mpt("0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47").unwrap();
    /// assert_eq!(mpt.mpt_issuance_id(), Some("0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47"));
    /// ```
    pub fn mpt_issuance_id(&self) -> Option<&str> {
        match self {
            Asset::Mpt { mpt_issuance_id } => Some(mpt_issuance_id),
            _ => None,
        }
    }

    /// Produce an [`Amount`] by pairing this asset with a value.
    ///
    /// # Example
    /// ```rust
    /// use xrpl::types::Asset;
    /// let asset = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap();
    /// let amount = asset.amount_with("100").unwrap();
    /// ```
    pub fn amount_with<V: Into<String>>(
        &self,
        value: V,
    ) -> Result<Amount, ValidationError> {
        let value = value.into();
        match self {
            Asset::Xrp { .. } => {
                value.parse::<u64>().map_err(|_| {
                    ValidationError::InvalidAmount(format!(
                        "Failed to parse '{value}' as drops"
                    ))
                })?;
                Ok(Amount::Xrpl(value))
            }
            Asset::Token { currency, issuer } => {
                validate_amount_string(&value)?;
                Ok(Amount::IssuedCurrency {
                    value,
                    currency: currency.clone(),
                    issuer: issuer.clone(),
                })
            }
            Asset::Mpt { mpt_issuance_id } => {
                validate_amount_string(&value)?;
                let val = value.parse::<u64>().map_err(|_| {
                    ValidationError::InvalidAmount(format!(
                        "MPT value must be a non-negative integer: '{value}'"
                    ))
                })?;
                if val > 0x7FFFFFFFFFFFFFFF {
                    return Err(ValidationError::InvalidAmount(
                        "MPT value exceeds maximum allowed value".into(),
                    ));
                }
                Ok(Amount::Mpt {
                    value,
                    mpt_issuance_id: mpt_issuance_id.clone(),
                })
            }
        }
    }
}

impl TryFrom<Amount> for Asset {
    type Error = ValidationError;

    /// Strip the value from an [`Amount`], returning just the asset identity.
    ///
    /// # Example
    /// ```rust
    /// use xrpl::types::{Amount, Asset};
    /// let amount = Amount::issued_currency("100", "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap();
    /// let asset = Asset::try_from(amount).unwrap();
    /// ```
    fn try_from(amount: Amount) -> Result<Self, Self::Error> {
        match amount {
            Amount::Xrpl(_) => Ok(Asset::xrp()),
            Amount::IssuedCurrency { currency, issuer, .. } => {
                Ok(Asset::Token { currency, issuer })
            }
            Amount::Mpt { mpt_issuance_id, .. } => {
                Ok(Asset::Mpt { mpt_issuance_id })
            }
        }
    }
}

impl TryFrom<&Amount> for Asset {
    type Error = ValidationError;

    fn try_from(amount: &Amount) -> Result<Self, Self::Error> {
        match amount {
            Amount::Xrpl(_) => Ok(Asset::xrp()),
            Amount::IssuedCurrency { currency, issuer, .. } => {
                Ok(Asset::Token {
                    currency: currency.clone(),
                    issuer: issuer.clone(),
                })
            }
            Amount::Mpt { mpt_issuance_id, .. } => {
                Ok(Asset::Mpt { mpt_issuance_id: mpt_issuance_id.clone() })
            }
        }
    }
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Asset::Xrp { .. } => write!(f, "XRP"),
            Asset::Token { currency, issuer } => {
                write!(
                    f,
                    "{} ({})",
                    currency,
                    if issuer.len() > 10 {
                        format!(
                            "{}...{}",
                            &issuer[..6],
                            &issuer[issuer.len() - 4..]
                        )
                    } else {
                        issuer.clone()
                    }
                )
            }
            Asset::Mpt { mpt_issuance_id } => {
                write!(
                    f,
                    "MPT ({}...{})",
                    &mpt_issuance_id[..8],
                    &mpt_issuance_id[mpt_issuance_id.len() - 6..]
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_without_amounts() {
        const ISSUER: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";

        let xrp_asset = Asset::xrp();
        match xrp_asset {
            Asset::Xrp { currency } => assert_eq!(currency, "XRP"),
            _ => panic!("Wrong asset type"),
        }

        let token_asset = Asset::token("USD", ISSUER).unwrap();
        match token_asset {
            Asset::Token { currency, issuer } => {
                assert_eq!(currency, "USD");
                assert_eq!(issuer, ISSUER);
            }
            _ => panic!("Wrong asset type"),
        }

        let mpt_asset =
            Asset::mpt("0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47")
                .unwrap();
        match mpt_asset {
            Asset::Mpt { mpt_issuance_id } => {
                assert_eq!(
                    mpt_issuance_id,
                    "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47"
                );
            }
            _ => panic!("Wrong asset type"),
        }

        // Invalid: XRP is reserved
        assert!(Asset::token("XRP", ISSUER).is_err());
        // Invalid: short address
        assert!(Asset::token("USD", "rIssuer").is_err());
        // Invalid MPT issuance ID
        assert!(Asset::mpt("invalid").is_err());
    }

    #[test]
    fn test_try_from_amount() {
        const ISSUER: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";

        let xrp = drops!(1_000_000);
        assert_eq!(Asset::try_from(xrp).unwrap(), Asset::xrp());

        let token = Amount::issued_currency("100", "USD", ISSUER).unwrap();
        assert_eq!(
            Asset::try_from(&token).unwrap(),
            Asset::Token { currency: "USD".into(), issuer: ISSUER.into() }
        );
        assert_eq!(token.value(), "100");

        let mpt = Amount::mpt(
            "500",
            "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47",
        )
        .unwrap();
        assert_eq!(
            Asset::try_from(mpt).unwrap(),
            Asset::Mpt {
                mpt_issuance_id:
                    "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47".into()
            }
        );
    }

    #[test]
    fn test_amount_with() {
        const ISSUER: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";

        let xrp = Asset::xrp();
        let amount = xrp.amount_with("1000000").unwrap();
        assert_eq!(amount, drops!(1_000_000));
        assert!(xrp.amount_with("not_a_number").is_err());

        let token = Asset::token("USD", ISSUER).unwrap();
        let amount = token.amount_with("100.5").unwrap();
        assert_eq!(amount.value(), "100.5");
        assert_eq!(amount.currency(), "USD");

        let mpt =
            Asset::mpt("0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47")
                .unwrap();
        let amount = mpt.amount_with("9999").unwrap();
        assert_eq!(amount.value(), "9999");
        assert!(mpt.amount_with("not_int").is_err());
        // Max value exceeded
        assert!(mpt.amount_with("9223372036854775808").is_err());
    }

    #[test]
    fn test_asset_display() {
        // Test XRP asset
        let xrp_asset = Asset::xrp();
        assert_eq!(format!("{xrp_asset}"), "XRP");

        // Test token asset
        let token_asset =
            Asset::token("USD", "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH").unwrap();
        assert_eq!(format!("{token_asset}"), "USD (rN7n7o...fzRH)");

        // Test MPT asset
        let mpt_asset =
            Asset::mpt("0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47")
                .unwrap();
        assert_eq!(format!("{mpt_asset}"), "MPT (0000012F...1BED47)");
    }
}
