use std::fmt;
use serde::{Deserialize, Serialize};
use super::validation::{
    ValidationError, validate_address, validate_currency_code, validate_mpt_id,
    validate_amount_string,
};

/// Represents an amount of currency on the XRPL: XRP, tokens, or MPTs.
///
/// # Construction
///
/// Three layers, pick the one that fits the situation:
///
/// 1. **Literals in source code** — use the macros [`xrp!`], [`drops!`],
///    [`issued!`], [`mpt!`]. They panic on invalid input, which is fine for
///    constants the author controls.
/// 2. **Runtime / untrusted input** — use the fallible constructors
///    [`Amount::xrp`], [`Amount::drops`], [`Amount::issued_currency`],
///    [`Amount::mpt`]. They return a [`Result`] and validate every field.
/// 3. **Your own domain type** — implement [`From<MyType> for Amount`] once
///    and pass `MyType` directly to any builder method (they all accept
///    `impl Into<Amount>`). Implement [`TryFrom<Amount> for MyType`] to
///    recover your type from ledger responses.
///
/// [`xrp!`]: crate::xrp
/// [`drops!`]: crate::drops
/// [`issued!`]: crate::issued
/// [`mpt!`]: crate::mpt
/// [`From<MyType> for Amount`]: From
/// [`TryFrom<Amount> for MyType`]: TryFrom
///
/// # Examples
///
/// Create an XRP amount (1.5 XRP):
/// ```rust
/// use xrpl::types::Amount;
/// let amount = Amount::xrp("1.5").unwrap();
/// ```
///
/// Create a token amount (100 USD issued by rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh):
/// ```rust
/// use xrpl::types::Amount;
/// let amount = Amount::issued_currency("100", "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap();
/// ```
///
/// Create an MPT amount:
/// ```rust
/// use xrpl::types::Amount;
/// let amount = Amount::mpt("1000000", "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47").unwrap();
/// ```
///
/// Same amounts using the literal macros:
/// ```rust
/// use xrpl::{xrp, drops, issued, mpt};
/// let xrp_amount = xrp!(1.5);
/// let same_xrp = drops!(1_500_000);
/// let usd = issued!(100, "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
/// let token = mpt!(1_000_000, "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47");
/// ```
///
/// Interop with your own domain type by implementing [`From`] and [`TryFrom`].
/// This lets your type be passed wherever an `impl Into<Amount>` is expected
/// (such as builder methods), and lets you recover your type from an `Amount`
/// returned by the ledger:
///
/// ```rust
/// use xrpl::types::Amount;
///
/// /// A domain type holding USD cents as an unsigned integer.
/// struct Usd { cents: u64 }
///
/// const USD_ISSUER: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
///
/// impl From<Usd> for Amount {
///     fn from(usd: Usd) -> Self {
///         let dollars = format!("{}.{:02}", usd.cents / 100, usd.cents % 100);
///         Amount::issued_currency(dollars, "USD", USD_ISSUER).unwrap()
///     }
/// }
///
/// impl TryFrom<Amount> for Usd {
///     type Error = &'static str;
///     fn try_from(a: Amount) -> Result<Self, Self::Error> {
///         match a {
///             Amount::IssuedCurrency { value, currency, issuer }
///                 if currency == "USD" && issuer == USD_ISSUER =>
///             {
///                 let dollars: f64 = value.parse().map_err(|_| "bad value")?;
///                 Ok(Usd { cents: (dollars * 100.0).round() as u64 })
///             }
///             _ => Err("not a USD amount from the expected issuer"),
///         }
///     }
/// }
///
/// let amount: Amount = Usd { cents: 12_345 }.into();
/// let back: Usd = amount.try_into().unwrap();
/// assert_eq!(back.cents, 12_345);
/// ```
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Amount {
    /// XRP amount in drops (string format)
    Xrpl(String),
    /// Token amount (issued currency)
    IssuedCurrency {
        /// Numeric value as a string (supports scientific notation for tokens).
        value: String,
        /// 3-character standard currency code or 40-hex non-standard code.
        currency: String,
        /// r-address of the token issuer.
        issuer: String,
    },
    /// MPT amount (Multi-Purpose Token)
    Mpt {
        /// Token quantity as a decimal string (positive integer for MPTs).
        value: String,
        /// 48-character hex MPT issuance ID.
        mpt_issuance_id: String,
    },
}

impl Default for Amount {
    fn default() -> Self {
        Amount::Xrpl("0".into())
    }
}

// Converts a decimal XRP string (e.g. "1.5") to an integer drop count using
// pure integer arithmetic, avoiding f64 precision loss for large amounts.
fn xrp_str_to_drops(xrp_str: &str) -> Result<u64, ValidationError> {
    let err = || {
        ValidationError::InvalidAmount(format!(
            "Invalid XRP value: '{xrp_str}'"
        ))
    };

    let (int_str, frac_str) = match xrp_str.split_once('.') {
        Some((i, f)) => (i, f),
        None => (xrp_str, ""),
    };

    if int_str.is_empty() || !int_str.chars().all(|c| c.is_ascii_digit()) {
        return Err(err());
    }
    if !frac_str.chars().all(|c| c.is_ascii_digit()) {
        return Err(err());
    }
    if frac_str.len() > 6 {
        return Err(ValidationError::InvalidAmount(format!(
            "XRP value '{xrp_str}' has more than 6 decimal places (sub-drop precision)"
        )));
    }

    // Pad fractional part to exactly 6 digits (left-aligned, trailing zeros)
    let frac_drops: u64 = format!("{frac_str:0<6}").parse().unwrap();

    let int_part: u64 = int_str.parse().map_err(|_| {
        ValidationError::InvalidAmount(format!(
            "XRP value too large: '{xrp_str}'"
        ))
    })?;

    int_part
        .checked_mul(1_000_000)
        .and_then(|i| i.checked_add(frac_drops))
        .ok_or_else(|| {
            ValidationError::InvalidAmount(format!(
                "XRP value '{xrp_str}' exceeds maximum drop count"
            ))
        })
}

impl Amount {
    /// Create XRP amount from XRP value (converts to drops).
    ///
    /// Accepts up to 6 decimal places (1 drop = 0.000001 XRP). More than 6
    /// decimal places is an error because sub-drop precision cannot be represented.
    pub fn xrp<T: Into<String>>(value: T) -> Result<Self, ValidationError> {
        let value = value.into();
        let drops = xrp_str_to_drops(&value)?;
        Ok(Amount::Xrpl(drops.to_string()))
    }

    /// Create XRP amount from drops (string format)
    pub fn drops<T: Into<String>>(value: T) -> Result<Self, ValidationError> {
        let value = value.into();
        value.parse::<u64>().map_err(|_| {
            ValidationError::InvalidAmount(format!(
                "Failed to parse '{value}' as drops"
            ))
        })?;
        Ok(Amount::Xrpl(value))
    }

    /// Create issued currency (token) amount
    pub fn issued_currency<V, C, I>(
        value: V,
        currency: C,
        issuer: I,
    ) -> Result<Self, ValidationError>
    where
        V: Into<String>,
        C: Into<String>,
        I: Into<String>,
    {
        let value = value.into();
        let currency = currency.into();
        let issuer = issuer.into();

        validate_currency_code(&currency, false)?;
        validate_address(&issuer)?;
        validate_amount_string(&value)?;

        if let Ok(val) = value.parse::<f64>() {
            if !val.is_finite() {
                return Err(ValidationError::InvalidAmount(format!(
                    "Invalid currency value: '{value}'"
                )));
            }
        } else {
            return Err(ValidationError::InvalidAmount(format!(
                "Invalid currency value: '{value}'"
            )));
        }

        Ok(Amount::IssuedCurrency { value, currency, issuer })
    }

    /// Create MPT (Multi-Purpose Token) amount
    pub fn mpt<V, I>(
        value: V,
        mpt_issuance_id: I,
    ) -> Result<Self, ValidationError>
    where
        V: Into<String>,
        I: Into<String>,
    {
        let value = value.into();
        let mpt_issuance_id = mpt_issuance_id.into();

        // Validate MPT issuance ID
        validate_mpt_id(&mpt_issuance_id)?;

        // Validate amount format
        validate_amount_string(&value)?;

        // MPT values must be positive integers
        let val = value.parse::<u64>().map_err(|_| {
            ValidationError::InvalidAmount(format!(
                "MPT value must be a positive integer: '{value}'"
            ))
        })?;

        // Check maximum value (0x7FFFFFFFFFFFFFFF)
        if val > 0x7FFFFFFFFFFFFFFF {
            return Err(ValidationError::InvalidAmount(
                "MPT value exceeds maximum allowed value".into(),
            ));
        }

        Ok(Amount::Mpt { value, mpt_issuance_id })
    }

    /// Returns the amount value as a string slice.
    pub fn value(&self) -> &str {
        match self {
            Amount::Xrpl(value) => value,
            Amount::IssuedCurrency { value, .. } => value,
            Amount::Mpt { value, .. } => value,
        }
    }

    /// Returns the currency code. Returns `"XRP"` for XRP amounts and an empty string for MPTs.
    /// Returns the currency code. Returns `"XRP"` for XRP amounts and an empty string for MPTs.
    ///
    /// # Example
    /// ```rust
    /// use xrpl::{xrp, issued};
    /// assert_eq!(xrp!(1.5).currency(), "XRP");
    /// let usd = issued!(100, "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    /// assert_eq!(usd.currency(), "USD");
    /// ```
    pub fn currency(&self) -> &str {
        match self {
            Amount::Xrpl(_) => "XRP",
            Amount::IssuedCurrency { currency, .. } => currency,
            Amount::Mpt { .. } => "", // MPTs don't have currency codes
        }
    }

    /// Returns the issuer address if this is an issued currency amount, otherwise `None`.
    ///
    /// # Example
    /// ```rust
    /// use xrpl::issued;
    /// let usd = issued!(100, "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    /// assert_eq!(usd.issuer(), Some("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"));
    /// ```
    pub fn issuer(&self) -> Option<&str> {
        match self {
            Amount::IssuedCurrency { issuer, .. } => Some(issuer),
            _ => None,
        }
    }

    /// Returns the MPT issuance ID if this is an MPT amount, otherwise `None`.
    ///
    /// # Example
    /// ```rust
    /// use xrpl::mpt;
    /// let token = mpt!(100, "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47");
    /// assert_eq!(token.mpt_issuance_id(), Some("0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47"));
    /// ```
    pub fn mpt_issuance_id(&self) -> Option<&str> {
        match self {
            Amount::Mpt { mpt_issuance_id, .. } => Some(mpt_issuance_id),
            _ => None,
        }
    }

    /// Returns the amount in drops as a `u64`. Returns `None` for issued currency and MPT amounts.
    pub fn to_drops(&self) -> Option<u64> {
        match self {
            Amount::Xrpl(value) => value.parse().ok(),
            Amount::IssuedCurrency { .. } | Amount::Mpt { .. } => None,
        }
    }

    /// Returns the raw drops string for XRP amounts. Returns `None` for issued currency and MPT amounts.
    pub fn as_drops(&self) -> Option<&str> {
        match self {
            Amount::Xrpl(value) => Some(value),
            Amount::IssuedCurrency { .. } | Amount::Mpt { .. } => None,
        }
    }

    /// Returns the amount as a decimal. For XRP, converts drops to XRP units. Returns `None` for MPTs.
    pub fn to_decimal(&self) -> Option<f64> {
        match self {
            Amount::Xrpl(value) => value
                .parse::<u64>()
                .ok()
                .map(|drops| drops as f64 / 1_000_000.0),
            Amount::IssuedCurrency { value, .. } => value.parse().ok(),
            Amount::Mpt { .. } => None, // MPTs are always integers
        }
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Amount::Xrpl(drops) => {
                // Convert drops to XRP for display
                if let Ok(drops_val) = drops.parse::<u64>() {
                    let xrp = drops_val as f64 / 1_000_000.0;
                    write!(f, "{xrp} XRP")
                } else {
                    write!(f, "{drops} drops")
                }
            }
            Amount::IssuedCurrency { value, currency, issuer } => {
                write!(
                    f,
                    "{} {} ({})",
                    value,
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
            Amount::Mpt { value, mpt_issuance_id } => {
                write!(
                    f,
                    "{} MPT ({}...{})",
                    value,
                    &mpt_issuance_id[..8],
                    &mpt_issuance_id[mpt_issuance_id.len() - 6..]
                )
            }
        }
    }
}

/// Create an XRP Amount from a float or integer value (in units of XRP).
///
/// # Example
///
/// ```rust
/// use xrpl::xrp;
/// use xrpl::types::Amount;
/// fn amount() {
///     let amount = xrp!(1.5); // 1.5 XRP
///     let amount_from_str = xrp!("1.5");
/// }
/// ```
#[macro_export]
macro_rules! xrp {
    ($amount:expr) => {
        $crate::types::Amount::xrp($amount.to_string()).unwrap()
    };
}

/// Create an XRP Amount from a value in drops (1 XRP = 1,000,000 drops).
///
/// # Example
///
/// ```rust
/// use xrpl::drops;
/// use xrpl::types::Amount;
/// fn amount() {
///     let amount = drops!(1_000_000); // 1 XRP
///     let amount_from_str = drops!("1000000");
/// }
/// ```
#[macro_export]
macro_rules! drops {
    ($amount:expr) => {
        $crate::types::Amount::drops($amount.to_string()).unwrap()
    };
}

/// Create an issued currency Amount from value, currency code, and issuer.
///
/// # Example
///
/// ```rust
/// use xrpl::issued;
/// use xrpl::types::Amount;
/// fn amount() {
///     let amount = issued!(100, "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
/// }
/// ```
#[macro_export]
macro_rules! issued {
    ($value:expr, $currency:expr, $issuer:expr) => {
        $crate::types::Amount::issued_currency(
            $value.to_string(),
            $currency,
            $issuer,
        )
        .unwrap()
    };
}

/// Create an MPT (Multi-Purpose Token) Amount from value and issuance ID.
///
/// # Example
///
/// ```rust
/// use xrpl::mpt;
/// use xrpl::types::Amount;
/// fn amount() {
///     let amount = mpt!(1_000_000, "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47");
/// }
/// ```
#[macro_export]
macro_rules! mpt {
    ($value:expr, $mpt_issuance_id:expr) => {
        $crate::types::Amount::mpt($value.to_string(), $mpt_issuance_id)
            .unwrap()
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversions() {
        let one_xrp = Amount::xrp("1").unwrap();
        let one_xrp_drops = Amount::drops("1000000").unwrap();
        let one_xrp_macro = xrp!(1.0);
        let one_xrp_drops_macro = drops!(1_000_000);

        assert_eq!(one_xrp, one_xrp_drops);
        assert_eq!(one_xrp_drops, one_xrp_macro);
        assert_eq!(one_xrp_macro, one_xrp_drops_macro);

        let half_xrp = Amount::xrp("0.5").unwrap();
        assert_eq!(half_xrp.to_drops().unwrap(), 500000);
        assert_eq!(half_xrp.to_decimal().unwrap(), 0.5);

        // Integer arithmetic: exactly correct for any value within u64 range
        let precise = Amount::xrp("1.123456").unwrap();
        assert_eq!(precise.to_drops().unwrap(), 1123456);

        let usd = Amount::issued_currency(
            "100.5",
            "USD",
            "rXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
        )
        .unwrap();

        assert_eq!(usd.currency(), "USD");
        assert_eq!(usd.value(), "100.5");

        let zero = Amount::default();
        assert_eq!(zero.to_drops().unwrap(), 0);

        let max_precision = Amount::xrp("1.999999").unwrap();
        assert_eq!(max_precision.to_drops().unwrap(), 1999999);

        // Sub-drop precision is rejected
        assert!(Amount::xrp("1.1234567").is_err());
    }

    #[test]
    fn test_mpt_amounts() {
        // Valid MPT
        let mpt = Amount::mpt(
            "1000000",
            "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47",
        )
        .unwrap();
        assert_eq!(mpt.value(), "1000000");

        // Test maximum value
        let max_mpt = Amount::mpt(
            "9223372036854775807",
            "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47",
        )
        .unwrap();
        assert_eq!(max_mpt.value(), "9223372036854775807");

        // Invalid: too large
        assert!(
            Amount::mpt(
                "18446744073709551616",
                "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47"
            )
            .is_err()
        );

        // Invalid: negative
        assert!(
            Amount::mpt(
                "-100",
                "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47"
            )
            .is_err()
        );

        // Invalid: non-integer
        assert!(
            Amount::mpt(
                "100.5",
                "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47"
            )
            .is_err()
        );

        // Invalid: wrong ID length
        assert!(Amount::mpt("100", "123").is_err());

        // Invalid: non-hex ID
        assert!(
            Amount::mpt(
                "100",
                "ZZZZ012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47"
            )
            .is_err()
        );
    }

    #[test]
    fn test_currency_code_validation() {
        const ISSUER: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";

        assert!(Amount::issued_currency("100", "USD", ISSUER).is_ok());
        assert!(Amount::issued_currency("100", "EUR", ISSUER).is_ok());
        assert!(Amount::issued_currency("100", "BTC", ISSUER).is_ok());
        assert!(Amount::issued_currency("100", "?!@", ISSUER).is_ok());

        // Invalid: XRP not allowed as issued currency code
        assert!(Amount::issued_currency("100", "XRP", ISSUER).is_err());

        // Invalid: wrong length
        assert!(Amount::issued_currency("100", "US", ISSUER).is_err());
        assert!(Amount::issued_currency("100", "USDT", ISSUER).is_err());

        // Valid nonstandard (40-char hex, not starting with 00)
        assert!(
            Amount::issued_currency(
                "100",
                "444F4C4C415259444F4F00000000000000000000",
                ISSUER,
            )
            .is_ok()
        );

        // Invalid: starts with 00
        assert!(
            Amount::issued_currency(
                "100",
                "004F4C4C415259444F4F00000000000000000000",
                ISSUER,
            )
            .is_err()
        );

        // Invalid: not hex
        assert!(
            Amount::issued_currency(
                "100",
                "ZZZF4C4C415259444F4F00000000000000000000",
                ISSUER,
            )
            .is_err()
        );

        // Invalid issuer address
        assert!(Amount::issued_currency("100", "USD", "rIssuer").is_err());
    }

    #[test]
    fn test_scientific_notation() {
        const ISSUER: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";

        assert!(Amount::issued_currency("1.23e11", "USD", ISSUER).is_ok());
        assert!(Amount::issued_currency("1.23E11", "USD", ISSUER).is_ok());
        assert!(Amount::issued_currency("1.5e-10", "USD", ISSUER).is_ok());
        assert!(Amount::issued_currency("-100.5", "USD", ISSUER).is_ok());
    }

    #[test]
    fn test_amount_display() {
        // Test XRP display (converts drops to XRP)
        let xrp_amount = Amount::drops("1000000").unwrap();
        assert_eq!(format!("{xrp_amount}"), "1 XRP");

        let xrp_fractional = Amount::drops("1500000").unwrap();
        assert_eq!(format!("{xrp_fractional}"), "1.5 XRP");

        // Test token display (with issuer truncation)
        let token_amount = Amount::issued_currency(
            "100.25",
            "USD",
            "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH",
        )
        .unwrap();
        assert_eq!(format!("{token_amount}"), "100.25 USD (rN7n7o...fzRH)");

        // Test MPT display
        let mpt_amount = Amount::mpt(
            "1000000",
            "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47",
        )
        .unwrap();
        assert_eq!(format!("{mpt_amount}"), "1000000 MPT (0000012F...1BED47)");
    }
}
