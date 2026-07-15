use thiserror::Error;
use super::Amount;

/// Errors returned by the input-validation helpers in this module.
///
/// # Examples
///
/// ```rust
/// use xrpl::types::validation::{ValidationError, validate_address};
/// let err = validate_address("not-an-address").unwrap_err();
/// assert!(matches!(err, ValidationError::InvalidAddress(_)));
/// ```
#[derive(Debug, Error, PartialEq, Clone)]
pub enum ValidationError {
    /// The supplied r-address or X-address is structurally invalid.
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    /// The supplied amount value is out of range or malformed.
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    /// The currency code fails length or character constraints.
    #[error("Invalid currency: {0}")]
    InvalidCurrency(String),
    /// The MPT issuance ID is not a valid 48-character hex string.
    #[error("Invalid MPT ID: {0}")]
    InvalidMptId(String),
    /// The invoice ID is not a valid 64-character hex string (32 bytes).
    #[error("Invalid InvoiceID: {0}")]
    InvalidInvoiceId(String),
    /// The domain is not valid hex.
    #[error("Invalid domain: {0}")]
    InvalidDomain(String),
    /// The email hash is not a valid 32-character hex string.
    #[error("Invalid email hash: {0}")]
    InvalidEmailHash(String),
    /// The message key is not valid hex.
    #[error("Invalid message key: {0}")]
    InvalidMessageKey(String),
}

/// Checks that `address` is a syntactically valid XRPL classic or X-address.
///
/// Classic addresses start with `r` and are 25-35 alphanumeric characters.
/// X-addresses start with `X` (mainnet) or `T` (testnet) and are exactly 47 characters.
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
                    "Invalid character in currency code: '{currency}'"
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

/// Validates invoice IDs (64-character hex strings representing 32 bytes).
pub fn validate_invoice_id(id: &str) -> Result<(), ValidationError> {
    if id.len() != 64 {
        return Err(ValidationError::InvalidInvoiceId(
            "InvoiceID must be exactly 64 characters (32 bytes)".into(),
        ));
    }
    if !id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError::InvalidInvoiceId(
            "InvoiceID must be hexadecimal".into(),
        ));
    }
    Ok(())
}

/// Validates domains (hex-encoded).
pub fn validate_domain(domain: &str) -> Result<(), ValidationError> {
    if !domain.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError::InvalidDomain(
            "Domain must be hex-encoded".into(),
        ));
    }
    Ok(())
}

/// Validates email hashes (32-character hex strings representing 16 bytes).
pub fn validate_email_hash(hash: &str) -> Result<(), ValidationError> {
    if hash.len() != 32 {
        return Err(ValidationError::InvalidEmailHash(
            "Email hash must be exactly 32 characters (16 bytes)".into(),
        ));
    }
    if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError::InvalidEmailHash(
            "Email hash must be hexadecimal".into(),
        ));
    }
    Ok(())
}

/// Validates message keys (hex-encoded, typically 66 characters).
pub fn validate_message_key(key: &str) -> Result<(), ValidationError> {
    if !key.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError::InvalidMessageKey(
            "Message key must be hexadecimal".into(),
        ));
    }
    // XRPL message keys are either 66 characters (secp256k1/ed25519) or empty.
    if !key.is_empty() && key.len() != 66 {
        return Err(ValidationError::InvalidMessageKey(
            "Message key must be 66 characters".into(),
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
            if value.is_empty() {
                return Err(ValidationError::InvalidAmount(
                    "XRP amount cannot be empty".into(),
                ));
            }
        }
        Amount::IssuedCurrency { value, currency, issuer } => {
            if value.is_empty() {
                return Err(ValidationError::InvalidAmount(
                    "Token value cannot be empty".into(),
                ));
            }

            validate_currency_code(currency, false)?;
            validate_address(issuer)?;
        }
        Amount::Mpt { value, mpt_issuance_id } => {
            if value.is_empty() {
                return Err(ValidationError::InvalidAmount(
                    "MPT value cannot be empty".into(),
                ));
            }

            validate_mpt_id(mpt_issuance_id)?;
            validate_amount_string(value)?;

            let val = value.parse::<u64>().map_err(|_| {
                ValidationError::InvalidAmount(
                    "MPT value must be a non-negative integer".into(),
                )
            })?;

            if val > 0x7FFFFFFFFFFFFFFF {
                return Err(ValidationError::InvalidAmount(
                    "MPT value exceeds maximum allowed value".into(),
                ));
            }
        }
    }
    Ok(())
}
