use std::ops::BitOr;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Transaction flags for [`MPTokenIssuanceCreate`].
///
/// Combine flags with `|` and pass the result to `with_flags` on the builder:
///
/// ```rust
/// use xrpl::types::MPTokenIssuanceCreateFlags as Flags;
///
/// let flags = Flags::CAN_TRANSFER | Flags::CAN_LOCK | Flags::CAN_CLAWBACK;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MPTokenIssuanceCreateFlags(pub u32);

impl MPTokenIssuanceCreateFlags {
    /// Issuer can lock individual balances or the entire issuance.
    pub const CAN_LOCK: Self = Self(0x0002);
    /// Holders must be authorized by the issuer before holding.
    pub const REQUIRE_AUTH: Self = Self(0x0004);
    /// Tokens can be held in escrow.
    pub const CAN_ESCROW: Self = Self(0x0008);
    /// Tokens can be traded on the DEX.
    pub const CAN_TRADE: Self = Self(0x0010);
    /// Holders can transfer tokens between accounts.
    pub const CAN_TRANSFER: Self = Self(0x0020);
    /// Issuer can claw back tokens from holders.
    pub const CAN_CLAWBACK: Self = Self(0x0040);

    /// Returns `true` if the given flag is set in this bitmask.
    pub fn has(self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }
}

impl BitOr for MPTokenIssuanceCreateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl From<u32> for MPTokenIssuanceCreateFlags {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<MPTokenIssuanceCreateFlags> for u32 {
    fn from(f: MPTokenIssuanceCreateFlags) -> u32 {
        f.0
    }
}

/// Transaction flags for [`MPTokenAuthorize`].
///
/// ```rust
/// use xrpl::types::MPTokenAuthorizeFlags;
///
/// let flags = MPTokenAuthorizeFlags::UNAUTHORIZE;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MPTokenAuthorizeFlags(pub u32);

impl MPTokenAuthorizeFlags {
    /// Removes the holder's authorization (opt-out).
    pub const UNAUTHORIZE: Self = Self(0x0001);

    /// Returns `true` if the given flag is set in this bitmask.
    pub fn has(self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }
}

impl BitOr for MPTokenAuthorizeFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl From<u32> for MPTokenAuthorizeFlags {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<MPTokenAuthorizeFlags> for u32 {
    fn from(f: MPTokenAuthorizeFlags) -> u32 {
        f.0
    }
}

/// Action for [`MPTokenIssuanceSet`] — lock or unlock an issuance or holder balance.
///
/// ```rust
/// use xrpl::types::MPTokenIssuanceSetAction;
///
/// let action = MPTokenIssuanceSetAction::Lock;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MPTokenIssuanceSetAction {
    /// Freeze the issuance or a specific holder's balance (`tfMPTLock`).
    Lock,
    /// Unfreeze the issuance or a specific holder's balance (`tfMPTUnlock`).
    Unlock,
}

impl From<MPTokenIssuanceSetAction> for u32 {
    fn from(a: MPTokenIssuanceSetAction) -> u32 {
        match a {
            MPTokenIssuanceSetAction::Lock => 0x00000001,
            MPTokenIssuanceSetAction::Unlock => 0x00000002,
        }
    }
}

/// Opts a holder in (or out) of an MPToken issuance.
///
/// A holder must authorize themselves before they can receive an MPT issuance.
/// When `tfMPTRequireAuth` is set on the issuance, the issuer uses this transaction
/// with `holder` populated to authorize individual accounts.
///
/// ```rust
/// use xrpl::types::transactions::mpt::MPTokenAuthorize;
/// let tx = MPTokenAuthorize {
///     mpt_issuance_id: "0000000024B5A7AE55A3019B1C7B38FBA04BEF0CEF2D6F48".to_string(),
///     holder: None, // omit when the holder self-authorizes
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MPTokenAuthorize {
    /// Identifier of the MPToken issuance.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mpt_issuance_id: String,
    /// Account to authorize; omit when the transaction submitter is self-authorizing.
    pub holder: Option<String>,
}

/// Creates a new Multi-Purpose Token (MPT) issuance class on the ledger.
///
/// The submitting account becomes the issuer. Flags passed via the transaction's
/// `Flags` field control transferability, clawback, and authorization requirements.
///
/// ```rust
/// use xrpl::types::transactions::mpt::MPTokenIssuanceCreate;
/// let tx = MPTokenIssuanceCreate {
///     asset_scale: Some(2),
///     maximum_amount: Some("1000000".to_string()),
///     mpt_metadata: None,
///     transfer_fee: Some(500), // 0.5%
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MPTokenIssuanceCreate {
    /// Number of decimal places (e.g. 2 means amounts are in hundredths).
    pub asset_scale: Option<u8>,
    /// Maximum number of tokens that may be distributed (UInt64 as a decimal string).
    pub maximum_amount: Option<String>,
    /// Hex-encoded metadata associated with the issuance.
    #[serde(rename = "MPTokenMetadata")]
    pub mpt_metadata: Option<String>,
    /// Transfer fee in units of 1/100,000 of a percent (0–50000).
    pub transfer_fee: Option<u16>,
}

/// Destroys an MPToken issuance that has an outstanding balance of zero.
///
/// Once destroyed, the issuance ID is permanently removed from the ledger.
///
/// ```rust
/// use xrpl::types::transactions::mpt::MPTokenIssuanceDestroy;
/// let tx = MPTokenIssuanceDestroy {
///     mpt_issuance_id: "0000000024B5A7AE55A3019B1C7B38FBA04BEF0CEF2D6F48".to_string(),
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MPTokenIssuanceDestroy {
    /// Identifier of the MPToken issuance to destroy.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mpt_issuance_id: String,
}

/// Locks or unlocks an MPToken issuance or a specific holder's balance.
///
/// Use flag `tfMPTLock` to lock and `tfMPTUnlock` to unlock. To target a single
/// holder's balance, provide the `holder` field; otherwise the entire issuance is affected.
///
/// ```rust
/// use xrpl::types::transactions::mpt::MPTokenIssuanceSet;
/// let tx = MPTokenIssuanceSet {
///     mpt_issuance_id: "0000000024B5A7AE55A3019B1C7B38FBA04BEF0CEF2D6F48".to_string(),
///     holder: None, // omit to lock/unlock the entire issuance
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MPTokenIssuanceSet {
    /// Identifier of the MPToken issuance to configure.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mpt_issuance_id: String,
    /// Specific holder account to lock or unlock; omit to affect the whole issuance.
    pub holder: Option<String>,
}
