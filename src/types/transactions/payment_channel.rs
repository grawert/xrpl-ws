use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::types::Amount;

/// Action for [`PaymentChannelClaim`] — close the channel or renew its settlement delay.
///
/// ```rust
/// use xrpl::types::PaymentChannelClaimAction;
///
/// let action = PaymentChannelClaimAction::Close;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentChannelClaimAction {
    /// Request to close the channel after the settlement delay (`tfClose`).
    Close,
    /// Reset the channel's expiry to now + settle delay (`tfRenew`).
    Renew,
}

impl From<PaymentChannelClaimAction> for u32 {
    fn from(a: PaymentChannelClaimAction) -> u32 {
        match a {
            PaymentChannelClaimAction::Close => 0x00020000,
            PaymentChannelClaimAction::Renew => 0x00040000,
        }
    }
}

/// Redeems a signed claim from a payment channel to receive XRP.
///
/// Either the sender or the recipient can submit this transaction. To close the
/// channel, set the `tfClose` flag. To renew the settlement delay, set `tfRenew`.
///
/// ```rust
/// use xrpl::types::{Amount, transactions::payment_channel::PaymentChannelClaim};
/// let tx = PaymentChannelClaim {
///     channel: "5DB01B7FFED6B67E6B0414DED11E051D2EE2B7619CE0EAA6286D67A3A4D5BDB3".to_string(),
///     amount: Some(Amount::Xrpl("1000000".to_string())),
///     balance: None,
///     credential_ids: None,
///     public_key: None,
///     signature: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PaymentChannelClaim {
    /// 256-bit ledger object ID of the payment channel.
    pub channel: String,
    /// Total XRP (drops) that the channel can pay out after this claim.
    pub amount: Option<Amount>,
    /// Total XRP (drops) delivered by the channel so far (cumulative).
    pub balance: Option<Amount>,
    /// Credential IDs used to satisfy deposit authorization on the destination.
    #[serde(rename = "CredentialIDs")]
    pub credential_ids: Option<Vec<String>>,
    /// Sender's secp256k1 or Ed25519 public key used to verify the signature.
    #[serde(rename = "PublicKey")]
    pub public_key: Option<String>,
    /// Sender's signature authorizing the claim amount.
    pub signature: Option<String>,
}

/// Opens a unidirectional XRP payment channel between two accounts.
///
/// The `settle_delay` enforces a waiting period after the channel is closed before
/// the sender can reclaim unclaimed XRP. The `public_key` must match the key used
/// to sign off-ledger claim messages.
///
/// ```rust
/// use xrpl::types::{Amount, transactions::payment_channel::PaymentChannelCreate};
/// let tx = PaymentChannelCreate {
///     amount: Amount::Xrpl("100000000".to_string()),
///     destination: "rRecipient".to_string(),
///     public_key: "ED...".to_string(),
///     settle_delay: 3600,
///     destination_tag: None,
///     cancel_after: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PaymentChannelCreate {
    /// Amount of XRP (drops) to fund the channel with.
    pub amount: Amount,
    /// Account that can receive XRP from this channel.
    pub destination: String,
    /// Sender's public key for verifying off-ledger claim signatures.
    #[serde(rename = "PublicKey")]
    pub public_key: String,
    /// Seconds the channel must remain open after a close request before XRP can be reclaimed.
    #[serde(rename = "SettleDelay")]
    pub settle_delay: u32,
    /// Destination tag for routing within the destination account.
    pub destination_tag: Option<u32>,
    /// Ripple-epoch time after which the channel can be closed by anyone.
    pub cancel_after: Option<u32>,
}

/// Adds more XRP to an open payment channel or extends its expiry.
///
/// Only the channel's source account can fund it. Optionally set or extend the
/// channel's expiration (must be after the current expiration if already set).
///
/// ```rust
/// use xrpl::types::{Amount, transactions::payment_channel::PaymentChannelFund};
/// let tx = PaymentChannelFund {
///     channel: "5DB01B7FFED6B67E6B0414DED11E051D2EE2B7619CE0EAA6286D67A3A4D5BDB3".to_string(),
///     amount: Amount::Xrpl("10000000".to_string()),
///     expiration: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PaymentChannelFund {
    /// 256-bit ledger object ID of the channel to fund.
    pub channel: String,
    /// Additional XRP (drops) to deposit into the channel.
    pub amount: Amount,
    /// New Ripple-epoch expiration time for the channel; must be later than the current expiry.
    pub expiration: Option<u32>,
}
