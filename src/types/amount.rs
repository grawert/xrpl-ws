use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Amount {
    Xrpl(String),
    IssuedCurrency { value: String, currency: String, issuer: String },
}

impl Default for Amount {
    fn default() -> Self {
        Amount::Xrpl("0".into())
    }
}

impl Amount {
    pub fn xrp<T: Into<String>>(value: T) -> Result<Self, String> {
        let value = value.into();
        let xrp = value
            .parse::<f64>()
            .map_err(|_| format!("Failed to parse '{}' as a number", value))?;
        let drops = (xrp * 1_000_000.0).round() as u64;
        Ok(Amount::Xrpl(drops.to_string()))
    }

    pub fn drops<T: Into<String>>(value: T) -> Result<Self, String> {
        let value = value.into();
        value
            .parse::<u64>()
            .map_err(|_| format!("Failed to parse '{}' as drops", value))?;
        Ok(Amount::Xrpl(value))
    }

    pub fn issued_currency<V, C, I>(
        value: V,
        currency: C,
        issuer: I,
    ) -> Result<Self, String>
    where
        V: Into<String>,
        C: Into<String>,
        I: Into<String>,
    {
        let value = value.into();
        value
            .parse::<f64>()
            .map_err(|_| format!("Invalid currency value: '{}'", value))?;
        Ok(Amount::IssuedCurrency {
            value,
            currency: currency.into(),
            issuer: issuer.into(),
        })
    }

    pub fn value(&self) -> &str {
        match self {
            Amount::Xrpl(value) => value,
            Amount::IssuedCurrency { value, .. } => value,
        }
    }

    pub fn currency(&self) -> &str {
        match self {
            Amount::Xrpl(_) => "XRP",
            Amount::IssuedCurrency { currency, .. } => currency,
        }
    }

    pub fn to_drops(&self) -> Option<u64> {
        match self {
            Amount::Xrpl(value) => value.parse().ok(),
            Amount::IssuedCurrency { .. } => None,
        }
    }

    pub fn to_decimal(&self) -> Option<f64> {
        match self {
            Amount::Xrpl(value) => value
                .parse::<u64>()
                .ok()
                .map(|drops| drops as f64 / 1_000_000.0),
            Amount::IssuedCurrency { value, .. } => value.parse().ok(),
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

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Amount::Xrpl(value) => write!(f, "{}", value),
            Amount::IssuedCurrency { value, .. } => write!(f, "{}", value),
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
        }
    }
}

#[macro_export]
macro_rules! xrp {
    ($amount:expr) => {
        Amount::from($amount as f64)
    };
}

#[macro_export]
macro_rules! drops {
    ($amount:expr) => {
        Amount::from($amount as u64)
    };
}

#[macro_export]
macro_rules! issued {
    ($value:expr, $currency:expr, $issuer:expr) => {
        Amount::issued_currency_infallible($value as f64, $currency, $issuer)
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
}
