use std::fmt;
use serde::{Deserialize, Serialize};
use super::validation::{ValidationError, validate_currency_code, validate_mpt_id};

/// Represents a currency/asset without a specific amount value
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Asset {
    Xrp { currency: String },
    Mpt { mpt_issuance_id: String },
    Token { currency: String, issuer: String },
}

impl Asset {
    /// Create XRP asset (without amount)
    pub fn xrp() -> Self {
        Asset::Xrp { currency: "XRP".to_string() }
    }

    /// Create token asset (without amount)
    pub fn token<C, I>(currency: C, issuer: I) -> Result<Self, ValidationError>
    where
        C: Into<String>,
        I: Into<String>,
    {
        let currency = currency.into();
        validate_currency_code(&currency, false)?;

        Ok(Asset::Token { currency, issuer: issuer.into() })
    }

    /// Create MPT asset (without amount)
    pub fn mpt<I>(mpt_issuance_id: I) -> Result<Self, ValidationError>
    where
        I: Into<String>,
    {
        let mpt_issuance_id = mpt_issuance_id.into();
        validate_mpt_id(&mpt_issuance_id)?;

        Ok(Asset::Mpt { mpt_issuance_id })
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
        // XRP asset
        let xrp_asset = Asset::xrp();
        match xrp_asset {
            Asset::Xrp { currency } => assert_eq!(currency, "XRP"),
            _ => panic!("Wrong asset type"),
        }

        // Token asset
        let token_asset = Asset::token("USD", "rIssuer").unwrap();
        match token_asset {
            Asset::Token { currency, issuer } => {
                assert_eq!(currency, "USD");
                assert_eq!(issuer, "rIssuer");
            }
            _ => panic!("Wrong asset type"),
        }

        // MPT asset
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

        // Invalid token asset
        assert!(Asset::token("XRP", "rIssuer").is_err());

        // Invalid MPT asset
        assert!(Asset::mpt("invalid").is_err());
    }

    #[test]
    fn test_asset_display() {
        // Test XRP asset
        let xrp_asset = Asset::xrp();
        assert_eq!(format!("{}", xrp_asset), "XRP");

        // Test token asset
        let token_asset =
            Asset::token("USD", "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH").unwrap();
        assert_eq!(format!("{}", token_asset), "USD (rN7n7o...fzRH)");

        // Test MPT asset
        let mpt_asset =
            Asset::mpt("0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47")
                .unwrap();
        assert_eq!(format!("{}", mpt_asset), "MPT (0000012F...1BED47)");
    }
}
