//! Ripple-epoch time utilities.
//!
//! The XRP Ledger measures time in seconds since the **Ripple Epoch**
//! (2000-01-01 00:00:00 UTC), stored as a [`u32`].  All transaction fields
//! that carry a timestamp — `Expiration`, `FinishAfter`, `CancelAfter` — use
//! this unit.
//!
//! UNIX timestamps (seconds since 1970-01-01) are offset by exactly
//! **946 684 800** seconds.  Passing a UNIX timestamp directly into one of
//! those fields silently sets the time ~30 years in the future; these helpers
//! make the conversion explicit.
//!
//! # Examples
//!
//! ```rust
//! use xrpl::time::{unix_to_ripple, ripple_to_unix, ripple_now};
//!
//! let unix_secs: u64 = 1_000_000_000; // 2001-09-09
//! let ripple = unix_to_ripple(unix_secs);
//! assert_eq!(ripple_to_unix(ripple), unix_secs);
//!
//! let now: u32 = ripple_now();
//! let expiry = now + 3600; // 1 hour from now in Ripple epoch
//! ```

/// Seconds between the UNIX epoch (1970-01-01) and the Ripple epoch (2000-01-01).
pub const RIPPLE_EPOCH_OFFSET: u64 = 946_684_800;

/// Converts a UNIX timestamp (seconds since 1970-01-01 UTC) to a Ripple epoch
/// timestamp (seconds since 2000-01-01 UTC).
///
/// # Panics
///
/// Panics if `unix_secs` is less than [`RIPPLE_EPOCH_OFFSET`] (i.e. before
/// 2000-01-01) or if the result overflows [`u32`].
pub fn unix_to_ripple(unix_secs: u64) -> u32 {
    (unix_secs - RIPPLE_EPOCH_OFFSET)
        .try_into()
        .expect("unix timestamp out of Ripple epoch u32 range")
}

/// Converts a Ripple epoch timestamp (seconds since 2000-01-01 UTC) to a UNIX
/// timestamp (seconds since 1970-01-01 UTC).
pub fn ripple_to_unix(ripple_secs: u32) -> u64 {
    u64::from(ripple_secs) + RIPPLE_EPOCH_OFFSET
}

/// Returns the current time as seconds since the Ripple epoch (2000-01-01 UTC).
pub fn ripple_now() -> u32 {
    unix_to_ripple(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock before UNIX epoch")
            .as_secs(),
    )
}
