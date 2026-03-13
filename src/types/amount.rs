use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use super::validation::{
    ValidationError, validate_currency_code, validate_mpt_id,
    validate_amount_string,
};

/// Represents an amount of currency on the XRPL: XRP, tokens, or MPTs.
///
/// # Examples
///
/// Create an XRP amount (1.5 XRP):
/// ```rust
/// use xrpl::types::Amount;
/// let amount = Amount::xrp("1.5").unwrap();
/// ```
///
/// Create a token amount (100 USD issued by rEXAMPLEissuer):
/// ```rust
/// use xrpl::types::Amount;
/// let amount = Amount::issued_currency("100", "USD", "rEXAMPLEissuer").unwrap();
/// ```
///
/// Create an MPT amount:
/// ```rust
/// use xrpl::types::Amount;
/// let amount = Amount::mpt("1000000", "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47").unwrap();
/// ```
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Amount {
    /// XRP amount in drops (string format)
    Xrpl(String),
    /// Token amount (issued currency)
    IssuedCurrency { value: String, currency: String, issuer: String },
    /// MPT amount (Multi-Purpose Token)
    Mpt { value: String, mpt_issuance_id: String },
}

impl Default for Amount {
    fn default() -> Self {
        Amount::Xrpl("0".into())
    }
}

impl Amount {
    /// Create XRP amount from XRP value (converts to drops)
    pub fn xrp<T: Into<String>>(value: T) -> Result<Self, ValidationError> {
        let value = value.into();
        let xrp = value.parse::<f64>().map_err(|_| {
            ValidationError::InvalidAmount(format!(
                "Failed to parse '{}' as a number",
                value
            ))
        })?;
        let drops = (xrp * 1_000_000.0).round() as u64;
        Ok(Amount::Xrpl(drops.to_string()))
    }

    /// Create XRP amount from drops (string format)
    pub fn drops<T: Into<String>>(value: T) -> Result<Self, ValidationError> {
        let value = value.into();
        value.parse::<u64>().map_err(|_| {
            ValidationError::InvalidAmount(format!(
                "Failed to parse '{}' as drops",
                value
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

        // Validate currency code (XRP not allowed for tokens)
        validate_currency_code(&currency, false)?;

        // Validate amount format
        validate_amount_string(&value)?;

        // Allow scientific notation and negative values for tokens
        // Just ensure it can be parsed as a finite number
        if let Ok(val) = value.parse::<f64>() {
            if !val.is_finite() {
                return Err(ValidationError::InvalidAmount(format!(
                    "Invalid currency value: '{}'",
                    value
                )));
            }
        } else {
            return Err(ValidationError::InvalidAmount(format!(
                "Invalid currency value: '{}'",
                value
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
                "MPT value must be a positive integer: '{}'",
                value
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

    pub fn value(&self) -> &str {
        match self {
            Amount::Xrpl(value) => value,
            Amount::IssuedCurrency { value, .. } => value,
            Amount::Mpt { value, .. } => value,
        }
    }

    pub fn currency(&self) -> &str {
        match self {
            Amount::Xrpl(_) => "XRP",
            Amount::IssuedCurrency { currency, .. } => currency,
            Amount::Mpt { .. } => "", // MPTs don't have currency codes
        }
    }

    pub fn to_drops(&self) -> Option<u64> {
        match self {
            Amount::Xrpl(value) => value.parse().ok(),
            Amount::IssuedCurrency { .. } | Amount::Mpt { .. } => None,
        }
    }

    pub fn as_drops(&self) -> Option<&str> {
        match self {
            Amount::Xrpl(value) => Some(value),
            Amount::IssuedCurrency { .. } | Amount::Mpt { .. } => None,
        }
    }

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
                    write!(f, "{} XRP", xrp)
                } else {
                    write!(f, "{} drops", drops)
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

impl From<f64> for Amount {
    fn from(xrp: f64) -> Self {
        let drops = (xrp * 1_000_000.0).round() as u64;
        Amount::Xrpl(drops.to_string())
    }
}

impl From<u64> for Amount {
    fn from(drops: u64) -> Self {
        Amount::Xrpl(drops.to_string())
    }
}

impl From<i64> for Amount {
    fn from(drops: i64) -> Self {
        Amount::Xrpl(drops.to_string())
    }
}

impl FromStr for Amount {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.chars().all(|c| c.is_ascii_digit()) {
            Ok(Amount::Xrpl(s.to_string()))
        } else {
            Err(format!("Cannot parse '{}' as Amount", s))
        }
    }
}

impl TryFrom<Amount> for u64 {
    type Error = String;

    fn try_from(amount: Amount) -> Result<Self, Self::Error> {
        match amount {
            Amount::Xrpl(value) => {
                value.parse().map_err(|_| "Invalid XRP amount".to_string())
            }
            Amount::IssuedCurrency { .. } => {
                Err("Cannot convert issued currency to u64".to_string())
            }
            Amount::Mpt { value, .. } => {
                value.parse().map_err(|_| "Invalid MPT amount".to_string())
            }
        }
    }
}

impl TryFrom<Amount> for f64 {
    type Error = String;

    fn try_from(amount: Amount) -> Result<Self, Self::Error> {
        match amount {
            Amount::Xrpl(value) => {
                let drops: u64 = value
                    .parse()
                    .map_err(|_| "Invalid XRP amount".to_string())?;
                Ok(drops as f64 / 1_000_000.0)
            }
            Amount::IssuedCurrency { value, .. } => {
                value.parse().map_err(|_| "Invalid currency amount".to_string())
            }
            Amount::Mpt { .. } => {
                Err("Cannot convert MPT to f64 (MPTs are integers only)"
                    .to_string())
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
/// fn main() {
///     let amount = xrp!(1.5); // 1.5 XRP
/// }
/// ```
#[macro_export]
macro_rules! xrp {
    ($amount:expr) => {
        Amount::from($amount as f64)
    };
}

/// Create an XRP Amount from a value in drops (1 XRP = 1,000,000 drops).
///
/// # Example
///
/// ```rust
/// use xrpl::drops;
/// use xrpl::types::Amount;
/// fn main() {
///     let amount = drops!(1_000_000); // 1 XRP
/// }
/// ```
#[macro_export]
macro_rules! drops {
    ($amount:expr) => {
        Amount::from($amount as u64)
    };
}

/// Create an issued currency Amount from value, currency code, and issuer.
///
/// # Example
///
/// ```rust
/// use xrpl::issued;
/// use xrpl::types::Amount;
/// fn main() {
///     let amount = issued!(100, "USD", "rEXAMPLEissuer");
/// }
/// ```
#[macro_export]
macro_rules! issued {
    ($value:expr, $currency:expr, $issuer:expr) => {
        Amount::issued_currency($value.to_string(), $currency, $issuer).unwrap()
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversions() {
        let amount1 = Amount::from(1000000i64);
        let amount2 = Amount::from(1.0f64);
        let amount3 = Amount::xrp("1").unwrap();
        let amount4 = Amount::drops("1000000").unwrap();
        let amount5 = xrp!(1.0);
        let amount6 = drops!(1000000);

        assert_eq!(amount1, amount2);
        assert_eq!(amount2, amount3);
        assert_eq!(amount4, amount5);
        assert_eq!(amount5, amount6);

        let half_xrp = Amount::from(0.5f64);
        assert_eq!(half_xrp.to_drops().unwrap(), 500000);
        assert_eq!(half_xrp.to_decimal().unwrap(), 0.5);

        let precise = Amount::from(1.123456f64);
        assert_eq!(precise.to_drops().unwrap(), 1123456);

        let usd = Amount::issued_currency(
            "100.5",
            "USD",
            "rXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
        )
        .unwrap();

        assert_eq!(usd.currency(), "USD");
        assert_eq!(usd.value(), "100.5");

        let drops: u64 = amount1.clone().try_into().unwrap();
        assert_eq!(drops, 1000000);

        let xrp_decimal: f64 = amount1.try_into().unwrap();
        assert_eq!(xrp_decimal, 1.0);

        let zero = Amount::from(0.0f64);
        assert_eq!(zero.to_drops().unwrap(), 0);

        let max_precision = Amount::from(1.999999f64);
        assert_eq!(max_precision.to_drops().unwrap(), 1999999);
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
        // Valid standard codes
        assert!(Amount::issued_currency("100", "USD", "rIssuer").is_ok());
        assert!(Amount::issued_currency("100", "EUR", "rIssuer").is_ok());
        assert!(Amount::issued_currency("100", "BTC", "rIssuer").is_ok());
        assert!(Amount::issued_currency("100", "?!@", "rIssuer").is_ok());

        // Invalid: XRP not allowed
        assert!(Amount::issued_currency("100", "XRP", "rIssuer").is_err());

        // Invalid: wrong length
        assert!(Amount::issued_currency("100", "US", "rIssuer").is_err());
        assert!(Amount::issued_currency("100", "USDT", "rIssuer").is_err());

        // Valid nonstandard (40-char hex, not starting with 00)
        assert!(
            Amount::issued_currency(
                "100",
                "444F4C4C415259444F4F00000000000000000000",
                "rIssuer"
            )
            .is_ok()
        );

        // Invalid: starts with 00
        assert!(
            Amount::issued_currency(
                "100",
                "004F4C4C415259444F4F00000000000000000000",
                "rIssuer"
            )
            .is_err()
        );

        // Invalid: not hex
        assert!(
            Amount::issued_currency(
                "100",
                "ZZZF4C4C415259444F4F00000000000000000000",
                "rIssuer"
            )
            .is_err()
        );
    }

    #[test]
    fn test_scientific_notation() {
        // Token amounts should support scientific notation
        assert!(Amount::issued_currency("1.23e11", "USD", "rIssuer").is_ok());
        assert!(Amount::issued_currency("1.23E11", "USD", "rIssuer").is_ok());
        assert!(Amount::issued_currency("1.5e-10", "USD", "rIssuer").is_ok());

        // Negative values should work for tokens
        assert!(Amount::issued_currency("-100.5", "USD", "rIssuer").is_ok());
    }

    #[test]
    fn test_amount_display() {
        // Test XRP display (converts drops to XRP)
        let xrp_amount = Amount::drops("1000000").unwrap();
        assert_eq!(format!("{}", xrp_amount), "1 XRP");

        let xrp_fractional = Amount::drops("1500000").unwrap();
        assert_eq!(format!("{}", xrp_fractional), "1.5 XRP");

        // Test token display (with issuer truncation)
        let token_amount = Amount::issued_currency(
            "100.25",
            "USD",
            "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH",
        )
        .unwrap();
        assert_eq!(format!("{}", token_amount), "100.25 USD (rN7n7o...fzRH)");

        // Test MPT display
        let mpt_amount = Amount::mpt(
            "1000000",
            "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47",
        )
        .unwrap();
        assert_eq!(
            format!("{}", mpt_amount),
            "1000000 MPT (0000012F...1BED47)"
        );
    }
}
