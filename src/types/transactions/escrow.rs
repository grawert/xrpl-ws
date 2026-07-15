use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::types::Amount;

/// Reclaims XRP from an expired escrow back to the owner.
///
/// Can be submitted by any account once the escrow's `CancelAfter` time has passed.
///
/// ```rust
/// use xrpl::types::transactions::escrow::EscrowCancel;
/// let tx = EscrowCancel {
///     owner: "rOwnerAccount".to_string(),
///     offer_sequence: 42,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct EscrowCancel {
    /// Account that created the escrow.
    pub owner: String,
    /// Sequence number of the `EscrowCreate` transaction that created the escrow.
    pub offer_sequence: u32,
}

/// Locks XRP in escrow with an optional time-lock or crypto-condition for release.
///
/// The escrowed XRP is released when `EscrowFinish` is submitted (with the correct
/// fulfillment if a condition was set) and after `FinishAfter` has passed.
///
/// ```rust
/// use xrpl::types::{Amount, transactions::escrow::EscrowCreate};
/// let tx = EscrowCreate {
///     amount: Amount::Xrpl("10000000".to_string()),
///     destination: "rRecipient".to_string(),
///     finish_after: Some(946_684_800 + 86_400), // one day after Ripple epoch
///     cancel_after: None,
///     condition: None,
///     destination_tag: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct EscrowCreate {
    /// Amount of XRP (in drops) to lock in escrow.
    pub amount: Amount,
    /// Account that receives the XRP when the escrow is finished.
    pub destination: String,
    /// Ripple-epoch time after which the escrow can be cancelled.
    pub cancel_after: Option<u32>,
    /// Ripple-epoch time after which the escrow can be finished.
    pub finish_after: Option<u32>,
    /// PREIMAGE-SHA-256 crypto-condition (hex-encoded) that must be fulfilled to release funds.
    pub condition: Option<String>,
    /// Destination tag for routing within the destination account.
    pub destination_tag: Option<u32>,
}

/// Releases escrowed XRP to the destination account.
///
/// If the escrow has a crypto-condition, both `condition` and `fulfillment` must be
/// provided. The transaction can only succeed after `FinishAfter` has passed.
///
/// ```rust
/// use xrpl::types::transactions::escrow::EscrowFinish;
/// let tx = EscrowFinish {
///     owner: "rOwnerAccount".to_string(),
///     offer_sequence: 42,
///     condition: None,
///     fulfillment: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct EscrowFinish {
    /// Account that created the escrow.
    pub owner: String,
    /// Sequence number of the `EscrowCreate` transaction that created the escrow.
    pub offer_sequence: u32,
    /// The PREIMAGE-SHA-256 crypto-condition (hex-encoded) originally set on the escrow.
    pub condition: Option<String>,
    /// The fulfillment (hex-encoded) that satisfies the condition.
    pub fulfillment: Option<String>,
}
