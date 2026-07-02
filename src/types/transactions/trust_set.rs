use std::ops::BitOr;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::types::Amount;

/// Transaction flags for [`TrustSet`].
///
/// ```rust
/// use xrpl::types::TrustSetFlags as Flags;
///
/// let flags = Flags::SET_NO_RIPPLE | Flags::SET_FREEZE;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrustSetFlags(pub u32);

impl TrustSetFlags {
    /// Authorize the trust line (requires `asfRequireAuth` on the issuer).
    pub const SET_AUTH: Self = Self(0x00010000);
    /// Block rippling through this trust line (recommended for holders).
    pub const SET_NO_RIPPLE: Self = Self(0x00020000);
    /// Re-enable rippling through this trust line.
    pub const CLEAR_NO_RIPPLE: Self = Self(0x00040000);
    /// Freeze this trust line; the counterparty cannot move the balance.
    pub const SET_FREEZE: Self = Self(0x00100000);
    /// Unfreeze this trust line.
    pub const CLEAR_FREEZE: Self = Self(0x00200000);

    /// Returns `true` if the given flag is set in this bitmask.
    pub fn has(self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }
}

impl BitOr for TrustSetFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl From<u32> for TrustSetFlags {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<TrustSetFlags> for u32 {
    fn from(f: TrustSetFlags) -> u32 {
        f.0
    }
}

/// Creates or modifies a trust line between the submitter and a currency issuer.
///
/// Setting `limit_amount` to zero with no outstanding balance closes the trust line.
/// Use the `tfSetNoRipple` / `tfClearNoRipple` flags to control rippling behavior.
///
/// ```rust
/// use xrpl::types::{Amount, transactions::trust_set::TrustSet};
/// let tx = TrustSet {
///     limit_amount: Amount::IssuedCurrency {
///         value: "1000".to_string(),
///         currency: "USD".to_string(),
///         issuer: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///     },
///     quality_in: None,
///     quality_out: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct TrustSet {
    /// Maximum amount of the issued currency the submitter is willing to hold; defines the trust line.
    pub limit_amount: Amount,
    /// Incoming exchange rate applied to balances flowing in through this trust line (billionths).
    pub quality_in: Option<u32>,
    /// Outgoing exchange rate applied to balances flowing out through this trust line (billionths).
    pub quality_out: Option<u32>,
}
