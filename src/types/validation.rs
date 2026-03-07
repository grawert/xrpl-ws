use crate::types::Amount;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum ValidationError {
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    #[error("Invalid currency: {0}")]
    InvalidCurrency(String),
}

pub fn validate_address(address: &str) -> Result<(), ValidationError> {
    if address.is_empty() {
        return Err(ValidationError::InvalidAddress(
            "Address cannot be empty".into(),
        ));
    }

    let first_char = address.chars().next();
    match first_char {
        Some('r') => {
            if !(25..=35).contains(&address.len()) {
                return Err(ValidationError::InvalidAddress(
                    "Classic address must be between 25 and 35 characters"
                        .into(),
                ));
            }
        }
        Some('X') | Some('T') => {
            if address.len() != 47 {
                return Err(ValidationError::InvalidAddress(
                    "X-address must be exactly 47 characters".into(),
                ));
            }
        }
        _ => {
            return Err(ValidationError::InvalidAddress(
                "Address must start with 'r', 'X', or 'T'".into(),
            ));
        }
    }

    if !address.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(ValidationError::InvalidAddress(
            "Address contains invalid characters".into(),
        ));
    }

    Ok(())
}

/// Validates currency codes for issued currencies.
pub fn validate_currency_code(currency: &str) -> Result<(), ValidationError> {
    if currency == "XRP" {
        // XRP is valid for native currency but we allow it for book subscriptions
        return Ok(());
    }

    let is_standard =
        currency.len() == 3 && currency.chars().all(|c| c.is_ascii_uppercase());
    let is_hex =
        currency.len() == 40 && currency.chars().all(|c| c.is_ascii_hexdigit());

    if !is_standard && !is_hex {
        return Err(ValidationError::InvalidCurrency(
            "Must be 3 uppercase characters or 40-char hex string".into(),
        ));
    }

    Ok(())
}

/// Validates XRP or Issued Currency amounts.
pub fn validate_amount(amount: &Amount) -> Result<(), ValidationError> {
    match amount {
        Amount::Xrpl(value) => {
            if value.is_empty() || value == "0" {
                return Err(ValidationError::InvalidAmount(
                    "XRP amount cannot be zero or empty".into(),
                ));
            }
        }
        Amount::IssuedCurrency { value, currency, issuer } => {
            if value.is_empty() || value == "0" {
                return Err(ValidationError::InvalidAmount(
                    "Token value cannot be zero or empty".into(),
                ));
            }

            // Use the currency validation function, but for amounts XRP is not allowed
            validate_currency_code(currency)?;
            if currency == "XRP" {
                return Err(ValidationError::InvalidCurrency(
                    "Currency code XRP is not allowed for issued currencies"
                        .into(),
                ));
            }

            validate_address(issuer)?;
        }
    }
    Ok(())
}
