use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::types::Amount;

/// Reclaims issued currency or MPT tokens from a holder's balance.
///
/// Only available to issuers whose token was created with clawback enabled.
/// For trust-line tokens, set the `issuer` sub-field of `amount` to the holder's address.
/// For MPTs, use the `holder` field instead.
///
/// ```rust
/// use xrpl::types::{Amount, transactions::clawback::Clawback};
/// let tx = Clawback {
///     amount: Amount::IssuedCurrency {
///         value: "100".to_string(),
///         currency: "USD".to_string(),
///         issuer: "rHolderAccount".to_string(),
///     },
///     holder: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Clawback {
    /// Amount to claw back; for trust-line tokens the `issuer` sub-field identifies the holder.
    pub amount: Amount,
    /// Holder account when clawing back MPT balances.
    pub holder: Option<String>,
}
