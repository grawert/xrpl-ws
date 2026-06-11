use std::ops::BitOr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::skip_serializing_none;

use crate::types::Amount;

/// An individual flag for [`Payment`] transactions.
///
/// Use with [`PaymentFlags`] to build or inspect the `Flags` bitmask.
///
/// # Examples
///
/// ```rust
/// use xrpl::types::{PaymentFlag, PaymentFlags};
///
/// // Building flags for a partial payment:
/// let flags = PaymentFlags::from(PaymentFlag::PartialPayment);
///
/// // Inspecting flags on a received transaction:
/// let raw: u32 = 0x00020000;
/// assert!(PaymentFlags::from(raw).has(PaymentFlag::PartialPayment));
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentFlag {
    /// Only use paths in the `paths` field; skip the default ripple path.
    NoRippleDirect,
    /// Allow delivery of less than the full `amount`; always check `delivered_amount` in metadata.
    PartialPayment,
    /// Only take paths where all offers meet or exceed the `send_max` quality ratio.
    LimitQuality,
    /// An unrecognized flag from a protocol amendment not yet reflected in this library.
    Unknown(u32),
}

impl PaymentFlag {
    /// The bitmask value for this flag as used in the `Flags` field.
    pub fn mask(self) -> u32 {
        match self {
            Self::NoRippleDirect => 0x00010000,
            Self::PartialPayment => 0x00020000,
            Self::LimitQuality => 0x00040000,
            Self::Unknown(v) => v,
        }
    }
}

impl From<PaymentFlag> for u32 {
    fn from(f: PaymentFlag) -> u32 {
        f.mask()
    }
}

impl BitOr for PaymentFlag {
    type Output = PaymentFlags;
    fn bitor(self, rhs: Self) -> PaymentFlags {
        PaymentFlags(self.mask() | rhs.mask())
    }
}

/// The `Flags` bitmask for a [`Payment`] transaction.
///
/// Use [`has`](Self::has) to check individual flags on incoming transactions,
/// and [`From<PaymentFlag>`] or [`BitOr`] to build a flags value for the builder.
///
/// # Examples
///
/// ```rust
/// use xrpl::types::{PaymentFlag, PaymentFlags};
///
/// // Combining flags for the builder:
/// let flags = PaymentFlag::PartialPayment | PaymentFlag::LimitQuality;
///
/// // Reading flags from a received transaction:
/// let flags = PaymentFlags::from(0x00020000_u32);
/// assert!(flags.has(PaymentFlag::PartialPayment));
/// assert!(!flags.has(PaymentFlag::LimitQuality));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PaymentFlags(u32);

impl PaymentFlags {
    /// Returns `true` if the given flag is set in this bitmask.
    pub fn has(self, flag: PaymentFlag) -> bool {
        let mask = flag.mask();
        mask != 0 && self.0 & mask != 0
    }

    /// The raw bitmask value.
    pub fn raw(self) -> u32 {
        self.0
    }
}

impl From<u32> for PaymentFlags {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<PaymentFlag> for PaymentFlags {
    fn from(f: PaymentFlag) -> Self {
        Self(f.mask())
    }
}

impl From<PaymentFlags> for u32 {
    fn from(f: PaymentFlags) -> u32 {
        f.0
    }
}

impl BitOr for PaymentFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOr<PaymentFlag> for PaymentFlags {
    type Output = Self;
    fn bitor(self, rhs: PaymentFlag) -> Self {
        Self(self.0 | rhs.mask())
    }
}

impl Serialize for PaymentFlags {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u32(self.0)
    }
}

impl<'de> Deserialize<'de> for PaymentFlags {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(Self(u32::deserialize(d)?))
    }
}

/// Sends XRP or issued currency to a destination account.
///
/// Supports direct XRP payments, issued-currency payments through trust lines,
/// and cross-currency payments via `paths`. Always verify `meta.delivered_amount`
/// rather than `amount` to guard against partial payment attacks.
///
/// ```rust
/// use xrpl::types::{Amount, transactions::payment::Payment};
/// let tx = Payment {
///     amount: Some(Amount::Xrpl("1000000".to_string())),
///     deliver_max: Some(Amount::Xrpl("1000000".to_string())),
///     destination: "rRecipient".to_string(),
///     deliver_min: None,
///     destination_tag: None,
///     invoice_id: None,
///     paths: None,
///     send_max: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Payment {
    /// The amount to deliver to the destination (for direct or single-currency payments).
    pub amount: Option<Amount>,
    /// Maximum amount to deliver; used in cross-currency or partial-payment scenarios.
    pub deliver_max: Option<Amount>,
    /// Minimum amount to deliver when `tfPartialPayment` is set.
    pub deliver_min: Option<Amount>,
    /// The account that receives the payment.
    pub destination: String,
    /// Destination tag for routing within the destination account.
    pub destination_tag: Option<u32>,
    /// 64-character hex invoice identifier for reconciliation.
    #[serde(rename = "InvoiceID")]
    pub invoice_id: Option<String>,
    /// Paths for cross-currency payments; each path is an ordered list of `PathStep` hops.
    pub paths: Option<Vec<Vec<PathStep>>>,
    /// Maximum amount the sender is willing to spend (for cross-currency payments).
    pub send_max: Option<Amount>,
}

/// Voids an uncashed check, removing it from the ledger.
///
/// Can be submitted by either the check sender or the intended recipient.
///
/// ```rust
/// use xrpl::types::transactions::payment::CheckCancel;
/// let tx = CheckCancel {
///     check_id: "838766BA2B995C00744175F69A1B11E32C3DBC40E64801A4056FCBD657F57334".to_string(),
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CheckCancel {
    /// Ledger object ID of the check to cancel.
    #[serde(rename = "CheckID")]
    pub check_id: String,
}

/// Redeems a check to receive funds from the check sender's account.
///
/// Exactly one of `amount` (exact delivery) or `deliver_min` (flexible delivery) must
/// be provided. Only the check's intended recipient can cash it.
///
/// ```rust
/// use xrpl::types::{Amount, transactions::payment::CheckCash};
/// let tx = CheckCash {
///     check_id: "838766BA2B995C00744175F69A1B11E32C3DBC40E64801A4056FCBD657F57334".to_string(),
///     amount: Some(Amount::Xrpl("1000000".to_string())),
///     deliver_min: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CheckCash {
    /// Ledger object ID of the check to cash.
    #[serde(rename = "CheckID")]
    pub check_id: String,
    /// Exact amount to receive; mutually exclusive with `deliver_min`.
    pub amount: Option<Amount>,
    /// Minimum amount to receive, allowing the ledger to deliver up to the check's `SendMax`.
    pub deliver_min: Option<Amount>,
}

/// Creates a deferred payment authorization (a "check") that the recipient may cash later.
///
/// Similar to a paper check: the sender pre-authorizes up to `send_max`, and the
/// recipient cashes it at any time before expiry.
///
/// ```rust
/// use xrpl::types::{Amount, transactions::payment::CheckCreate};
/// let tx = CheckCreate {
///     destination: "rRecipient".to_string(),
///     send_max: Amount::Xrpl("10000000".to_string()),
///     destination_tag: None,
///     expiration: None,
///     invoice_id: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CheckCreate {
    /// Account authorized to cash the check.
    pub destination: String,
    /// Maximum amount the sender is willing to pay when the check is cashed.
    pub send_max: Amount,
    /// Destination tag for routing within the destination account.
    pub destination_tag: Option<u32>,
    /// Ripple-epoch time after which the check can no longer be cashed.
    pub expiration: Option<u32>,
    /// 64-character hex invoice identifier for reconciliation.
    #[serde(rename = "InvoiceID")]
    pub invoice_id: Option<String>,
}

/// One intermediate hop in a cross-currency payment path.
///
/// Each step specifies the account (rippling through), currency, or issuer at that
/// point in the path. The combination of fields determines what type of hop it is.
///
/// ```rust
/// use xrpl::types::transactions::payment::PathStep;
/// let hop = PathStep {
///     account: Some("rIntermediaryAccount".to_string()),
///     currency: None,
///     issuer: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PathStep {
    /// Intermediate account to ripple through.
    pub account: Option<String>,
    /// Currency to convert into at this step.
    pub currency: Option<String>,
    /// Issuer of the currency at this step.
    pub issuer: Option<String>,
}

impl PathStep {
    /// Creates an empty path step. Chain `with_*` methods to populate the
    /// relevant fields for either an account-only hop or a currency hop.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the intermediate account.
    pub fn with_account(mut self, account: impl AsRef<str>) -> Self {
        self.account = Some(account.as_ref().to_string());
        self
    }

    /// Sets the currency code.
    pub fn with_currency(mut self, currency: impl AsRef<str>) -> Self {
        self.currency = Some(currency.as_ref().to_string());
        self
    }

    /// Sets the currency issuer.
    pub fn with_issuer(mut self, issuer: impl AsRef<str>) -> Self {
        self.issuer = Some(issuer.as_ref().to_string());
        self
    }
}
