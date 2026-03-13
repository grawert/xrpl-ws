use thiserror::Error;
use super::Amount;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum ValidationError {
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    #[error("Invalid currency: {0}")]
    InvalidCurrency(String),
    #[error("Invalid MPT ID: {0}")]
    InvalidMptId(String),
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

/// Validates currency codes
pub fn validate_currency_code(
    currency: &str,
    xrp_allowed: bool,
) -> Result<(), ValidationError> {
    if currency == "XRP" {
        if xrp_allowed {
            return Ok(());
        } else {
            return Err(ValidationError::InvalidCurrency(
                "Currency code 'XRP' is not allowed for tokens".into(),
            ));
        }
    }

    if currency.len() == 3 {
        // Standard currency code validation - allow alphanumeric and special chars
        for c in currency.chars() {
            if !c.is_ascii_alphanumeric() && !"?!@#$%^&*<>(){}[]|".contains(c) {
                return Err(ValidationError::InvalidCurrency(format!(
                    "Invalid character in currency code: '{}'",
                    currency
                )));
            }
        }
        Ok(())
    } else if currency.len() == 40 {
        // Nonstandard currency code (40-char hex)
        if !currency.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ValidationError::InvalidCurrency(
                "40-character currency code must be hexadecimal".into(),
            ));
        }
        // First 8 bits should NOT be 0x00 to prevent confusion with standard codes
        if currency.starts_with("00") {
            return Err(ValidationError::InvalidCurrency(
                "Nonstandard currency code should not start with '00'".into(),
            ));
        }
        Ok(())
    } else {
        Err(ValidationError::InvalidCurrency(format!(
            "Currency code must be 3 or 40 characters, got {}",
            currency.len()
        )))
    }
}

/// Validates MPT issuance IDs (48-character hex strings)
pub fn validate_mpt_id(mpt_id: &str) -> Result<(), ValidationError> {
    if mpt_id.len() != 48 {
        return Err(ValidationError::InvalidMptId(
            "MPT issuance ID must be 48 characters".into(),
        ));
    }

    if !mpt_id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError::InvalidMptId(
            "MPT issuance ID must be hexadecimal".into(),
        ));
    }

    Ok(())
}

/// Validates amount strings for issued currencies and MPTs
pub fn validate_amount_string(value: &str) -> Result<(), ValidationError> {
    if value.is_empty() {
        return Err(ValidationError::InvalidAmount(
            "Amount cannot be empty".into(),
        ));
    }

    // Check for valid numeric string
    if !value.chars().all(|c| {
        c.is_ascii_digit()
            || c == '.'
            || c == '-'
            || c == '+'
            || c == 'e'
            || c == 'E'
    }) {
        return Err(ValidationError::InvalidAmount(
            "Amount must be a valid numeric string".into(),
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

            // Validate currency code (XRP not allowed for issued currencies)
            validate_currency_code(currency, false)?;

            validate_address(issuer)?;
        }
        Amount::Mpt { value, mpt_issuance_id } => {
            if value.is_empty() || value == "0" {
                return Err(ValidationError::InvalidAmount(
                    "MPT value cannot be zero or empty".into(),
                ));
            }

            // Validate MPT issuance ID
            validate_mpt_id(mpt_issuance_id)?;

            // Validate MPT value format
            validate_amount_string(value)?;

            // MPT values must be positive integers
            if let Err(_) = value.parse::<u64>() {
                return Err(ValidationError::InvalidAmount(
                    "MPT value must be a positive integer".into(),
                ));
            }

            // Check maximum value for MPTs
            if let Ok(val) = value.parse::<u64>() {
                if val > 0x7FFFFFFFFFFFFFFF {
                    return Err(ValidationError::InvalidAmount(
                        "MPT value exceeds maximum allowed value".into(),
                    ));
                }
            }
        }
    }
    Ok(())
}
