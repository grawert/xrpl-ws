use std::ops::BitOr;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::types::Amount;

/// Transaction flags for [`OfferCreate`].
///
/// ```rust
/// use xrpl::types::OfferCreateFlags as Flags;
///
/// let flags = Flags::PASSIVE | Flags::SELL;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OfferCreateFlags(pub u32);

impl OfferCreateFlags {
    /// Do not consume offers that cross this one; post only.
    pub const PASSIVE: Self = Self(0x00010000);
    /// Fill as much as possible; cancel any unfilled remainder immediately.
    pub const IMMEDIATE_OR_CANCEL: Self = Self(0x00020000);
    /// Fill the full amount or cancel the entire offer.
    pub const FILL_OR_KILL: Self = Self(0x00040000);
    /// Exchange `taker_gets` for `taker_pays` at the market rate (sell mode).
    pub const SELL: Self = Self(0x00080000);

    /// Returns `true` if the given flag is set in this bitmask.
    pub fn has(self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }
}

impl BitOr for OfferCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl From<u32> for OfferCreateFlags {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<OfferCreateFlags> for u32 {
    fn from(f: OfferCreateFlags) -> u32 {
        f.0
    }
}

/// Cancels an open limit order on the XRPL decentralized exchange by sequence number.
///
/// ```rust
/// use xrpl::types::transactions::offer::OfferCancel;
/// let tx = OfferCancel { offer_sequence: 42 };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct OfferCancel {
    /// Sequence number of the `OfferCreate` transaction that placed the order.
    pub offer_sequence: u32,
}

/// Places a limit order on the XRPL decentralized exchange.
///
/// The order is filled immediately against existing offers in the order book at
/// as-good-or-better rates. Any unfilled remainder is placed on the book unless
/// the `tfImmediateOrCancel` or `tfFillOrKill` flags are set.
///
/// ```rust
/// use xrpl::types::{Amount, transactions::offer::OfferCreate};
/// let tx = OfferCreate {
///     taker_gets: Amount::Xrpl("1000000".to_string()),
///     taker_pays: Amount::IssuedCurrency {
///         value: "1".to_string(),
///         currency: "USD".to_string(),
///         issuer: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///     },
///     expiration: None,
///     offer_sequence: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct OfferCreate {
    /// Ripple-epoch time after which the offer is automatically invalidated.
    pub expiration: Option<u32>,
    /// Sequence number of an existing offer to cancel when this offer is placed.
    pub offer_sequence: Option<u32>,
    /// Amount the taker receives (what the submitter gives up).
    pub taker_gets: Amount,
    /// Amount the taker pays (what the submitter receives).
    pub taker_pays: Amount,
}
