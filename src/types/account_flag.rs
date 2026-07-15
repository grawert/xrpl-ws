use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The active account flags as returned by `account_info`.
///
/// Wraps the raw `Flags` bitmask from `AccountRoot` and provides typed access
/// via [`has`](Self::has). The raw value is preserved so that bits from unknown
/// amendments are never silently discarded.
///
/// # Examples
///
/// ```rust
/// use xrpl::types::{AccountFlag, AccountFlags};
///
/// let flags = AccountFlags::from(0x00820000_u32); // DefaultRipple + RequireDest
/// assert!(flags.has(AccountFlag::DefaultRipple));
/// assert!(flags.has(AccountFlag::RequireDest));
/// assert!(!flags.has(AccountFlag::DisableMaster));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AccountFlags(u32);

impl AccountFlags {
    /// Returns `true` if the given flag is set.
    ///
    /// Always returns `false` for flags that have no ledger-state representation
    /// (`AccountTxnId`, `AuthorizedNftokenMinter`).
    pub fn has(self, flag: AccountFlag) -> bool {
        let mask = flag.lsf_mask();
        mask != 0 && self.0 & mask != 0
    }

    /// The raw bitmask as received from the ledger.
    pub fn raw(self) -> u32 {
        self.0
    }
}

impl From<u32> for AccountFlags {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<AccountFlags> for u32 {
    fn from(f: AccountFlags) -> u32 {
        f.0
    }
}

impl Serialize for AccountFlags {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u32(self.0)
    }
}

impl<'de> Deserialize<'de> for AccountFlags {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(Self(u32::deserialize(d)?))
    }
}

/// An account-level flag used in [`AccountSet`](crate::types::transactions::account::AccountSet) transactions.
///
/// Each variant encodes both representations: the `asf*` integer index used in
/// `SetFlag`/`ClearFlag` fields (via [`asf_index`](Self::asf_index)) and the
/// `lsf*` bitmask used in `account_info` `Flags` fields
/// (via [`lsf_mask`](Self::lsf_mask)).
///
/// # Examples
///
/// ```rust
/// use xrpl::types::AccountFlag;
///
/// assert_eq!(AccountFlag::RequireDest.asf_index(), 1);
/// assert_eq!(AccountFlag::RequireDest.lsf_mask(), 0x00020000);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountFlag {
    /// Require a destination tag on all incoming transactions.
    RequireDest,
    /// Require authorization before users can hold this account's issued tokens.
    RequireAuth,
    /// Advisory: request that senders do not send XRP to this account.
    DisallowXrp,
    /// Disable the master key pair; a regular key or signer list must exist first.
    DisableMaster,
    /// Track the ID of this account's most recent transaction.
    AccountTxnId,
    /// Permanently give up the ability to freeze individual trust lines or apply Global Freeze.
    NoFreeze,
    /// Freeze all assets issued by this account.
    GlobalFreeze,
    /// Enable rippling on trust lines by default; required for token issuers.
    DefaultRipple,
    /// Enable Deposit Authorization; only pre-authorized senders can deposit.
    DepositAuth,
    /// Authorize another account to mint NFTokens on behalf of this account.
    AuthorizedNftokenMinter,
    /// Block incoming NFTokenOffer objects directed at this account.
    DisallowIncomingNftokenOffer,
    /// Block incoming Check objects directed at this account.
    DisallowIncomingCheck,
    /// Block incoming PayChannel objects directed at this account.
    DisallowIncomingPayChan,
    /// Block incoming TrustLine objects directed at this account.
    DisallowIncomingTrustline,
    /// Allow the account to claw back tokens it has issued. Irreversible once set.
    AllowTrustLineClawback,
    /// Allow trust line tokens issued by this account to be held in escrow. Irreversible once set.
    AllowTrustLineLocking,
    /// An unrecognized flag value from a protocol amendment not yet reflected in this library.
    Unknown(u32),
}

impl AccountFlag {
    /// The `asf*` integer index used in [`AccountSet`](crate::types::transactions::account::AccountSet) `SetFlag`/`ClearFlag` fields.
    pub fn asf_index(self) -> u32 {
        match self {
            Self::RequireDest => 1,
            Self::RequireAuth => 2,
            Self::DisallowXrp => 3,
            Self::DisableMaster => 4,
            Self::AccountTxnId => 5,
            Self::NoFreeze => 6,
            Self::GlobalFreeze => 7,
            Self::DefaultRipple => 8,
            Self::DepositAuth => 9,
            Self::AuthorizedNftokenMinter => 10,
            Self::DisallowIncomingNftokenOffer => 12,
            Self::DisallowIncomingCheck => 13,
            Self::DisallowIncomingPayChan => 14,
            Self::DisallowIncomingTrustline => 15,
            Self::AllowTrustLineClawback => 16,
            Self::AllowTrustLineLocking => 17,
            Self::Unknown(v) => v,
        }
    }

    /// The `lsf*` bitmask for checking this flag in `account_info` `Flags` fields.
    ///
    /// Returns `0` for flags that have no ledger-state representation
    /// (`AccountTxnId`, `AuthorizedNftokenMinter`).
    pub fn lsf_mask(self) -> u32 {
        match self {
            Self::RequireDest => 0x00020000,
            Self::RequireAuth => 0x00040000,
            Self::DisallowXrp => 0x00080000,
            Self::DisableMaster => 0x00100000,
            Self::AccountTxnId => 0,
            Self::NoFreeze => 0x00200000,
            Self::GlobalFreeze => 0x00400000,
            Self::DefaultRipple => 0x00800000,
            Self::DepositAuth => 0x01000000,
            Self::AuthorizedNftokenMinter => 0,
            Self::DisallowIncomingNftokenOffer => 0x04000000,
            Self::DisallowIncomingCheck => 0x08000000,
            Self::DisallowIncomingPayChan => 0x10000000,
            Self::DisallowIncomingTrustline => 0x20000000,
            Self::AllowTrustLineClawback => 0x80000000,
            Self::AllowTrustLineLocking => 0x40000000,
            Self::Unknown(_) => 0,
        }
    }

    /// Converts an AccountSet flag index (`asf` code) to the corresponding
    /// [`AccountFlag`] variant. Returns [`Unknown`](Self::Unknown) for unrecognized values.
    pub fn from_asf_index(v: u32) -> Self {
        match v {
            1 => Self::RequireDest,
            2 => Self::RequireAuth,
            3 => Self::DisallowXrp,
            4 => Self::DisableMaster,
            5 => Self::AccountTxnId,
            6 => Self::NoFreeze,
            7 => Self::GlobalFreeze,
            8 => Self::DefaultRipple,
            9 => Self::DepositAuth,
            10 => Self::AuthorizedNftokenMinter,
            12 => Self::DisallowIncomingNftokenOffer,
            13 => Self::DisallowIncomingCheck,
            14 => Self::DisallowIncomingPayChan,
            15 => Self::DisallowIncomingTrustline,
            16 => Self::AllowTrustLineClawback,
            17 => Self::AllowTrustLineLocking,
            v => Self::Unknown(v),
        }
    }
}

impl From<u32> for AccountFlag {
    fn from(v: u32) -> Self {
        Self::from_asf_index(v)
    }
}

impl Serialize for AccountFlag {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u32(self.asf_index())
    }
}

impl<'de> Deserialize<'de> for AccountFlag {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(Self::from_asf_index(u32::deserialize(d)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_known_flags() {
        let flags = [
            AccountFlag::RequireDest,
            AccountFlag::DefaultRipple,
            AccountFlag::DisableMaster,
            AccountFlag::AllowTrustLineClawback,
            AccountFlag::AllowTrustLineLocking,
        ];
        for flag in flags {
            let idx = flag.asf_index();
            assert_eq!(AccountFlag::from_asf_index(idx), flag);
        }
    }

    #[test]
    fn test_unknown_roundtrip() {
        let flag = AccountFlag::Unknown(99);
        assert_eq!(flag.asf_index(), 99);
        assert_eq!(flag.lsf_mask(), 0);
        assert_eq!(AccountFlag::from_asf_index(99), AccountFlag::Unknown(99));
    }

    #[test]
    fn test_serialize_as_asf_index() {
        let v = serde_json::to_value(AccountFlag::RequireDest).unwrap();
        assert_eq!(v, serde_json::json!(1));

        let v = serde_json::to_value(AccountFlag::DefaultRipple).unwrap();
        assert_eq!(v, serde_json::json!(8));
    }

    #[test]
    fn test_deserialize_from_asf_index() {
        let flag: AccountFlag =
            serde_json::from_value(serde_json::json!(1)).unwrap();
        assert_eq!(flag, AccountFlag::RequireDest);

        let flag: AccountFlag =
            serde_json::from_value(serde_json::json!(99)).unwrap();
        assert_eq!(flag, AccountFlag::Unknown(99));
    }

    #[test]
    fn test_no_lsf_mask_for_stateless_flags() {
        assert_eq!(AccountFlag::AccountTxnId.lsf_mask(), 0);
        assert_eq!(AccountFlag::AuthorizedNftokenMinter.lsf_mask(), 0);
    }
}
