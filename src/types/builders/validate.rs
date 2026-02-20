use crate::types::Amount;
use super::BuildError;

pub fn validate_address(address: &str, field: &str) -> Result<(), BuildError> {
    if address.is_empty()
        || !address.starts_with('r')
        || address.len() < 25
        || address.len() > 34
    {
        return Err(BuildError::InvalidField(format!(
            "Invalid {field} address"
        )));
    }
    Ok(())
}

pub fn validate_amount(amount: &Amount) -> Result<(), BuildError> {
    match amount {
        Amount::Xrpl(value) => {
            if value.is_empty() || value == "0" {
                return Err(BuildError::InvalidAmount(
                    "XRP amount cannot be zero or empty".to_string(),
                ));
            }
            Ok(())
        }
        Amount::IssuedCurrency { value, currency, issuer } => {
            if value.is_empty() || value == "0" {
                return Err(BuildError::InvalidAmount(
                    "Token value cannot be zero or empty".to_string(),
                ));
            }
            if currency.len() != 3
                || !currency.chars().all(|c| c.is_ascii_uppercase())
            {
                return Err(BuildError::InvalidField(
                    "Currency must be exactly 3 uppercase ASCII characters"
                        .to_string(),
                ));
            }
            if currency == "XRP" {
                return Err(BuildError::InvalidField(
                    "Currency code XRP is not allowed for issued currencies"
                        .to_string(),
                ));
            }
            validate_address(issuer, "issuer")
        }
    }
}
