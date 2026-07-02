use std::ops::BitOr;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::types::{Amount, XChainBridge};

/// Transaction flags for [`XChainModifyBridge`].
///
/// ```rust
/// use xrpl::types::XChainModifyBridgeFlags as Flags;
///
/// let flags = Flags::CLEAR_ACCOUNT_CREATE_AMOUNT;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XChainModifyBridgeFlags(pub u32);

impl XChainModifyBridgeFlags {
    /// Clears the `MinAccountCreateAmount` field on the bridge.
    pub const CLEAR_ACCOUNT_CREATE_AMOUNT: Self = Self(0x00010000);

    /// Returns `true` if the given flag is set in this bitmask.
    pub fn has(self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }
}

impl BitOr for XChainModifyBridgeFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl From<u32> for XChainModifyBridgeFlags {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<XChainModifyBridgeFlags> for u32 {
    fn from(f: XChainModifyBridgeFlags) -> u32 {
        f.0
    }
}

/// Funds the creation of an account on the destination chain via a cross-chain bridge.
///
/// Used when the target account does not yet exist on the other chain. Witness servers
/// observe this transaction and attest with `XChainAddAccountCreateAttestation`.
///
/// ```rust
/// use xrpl::types::{Amount, Asset, XChainBridge, transactions::xchain::XChainAccountCreateCommit};
/// let tx = XChainAccountCreateCommit {
///     amount: Amount::Xrpl("20000000".to_string()),
///     destination: "rNewAccount".to_string(),
///     signature_reward: Amount::Xrpl("100000000".to_string()),
///     xchain_bridge: XChainBridge {
///         locking_chain_door: "rLockDoor".to_string(),
///         locking_chain_issue: Asset::xrp(),
///         issuing_chain_door: "rIssueDoor".to_string(),
///         issuing_chain_issue: Asset::xrp(),
///     },
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct XChainAccountCreateCommit {
    /// XRP (or token) amount to send to fund the new account on the other chain.
    pub amount: Amount,
    /// Account to create on the destination chain.
    pub destination: String,
    /// Reward paid to witness servers for attesting this transaction.
    pub signature_reward: Amount,
    /// Bridge configuration identifying the two chains and door accounts.
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge,
}

/// Witness server attestation that an `XChainAccountCreateCommit` occurred on the source chain.
///
/// Submitted by each witness server individually. Once a quorum of attestations is
/// collected, the destination-chain account creation is finalized.
///
/// ```rust
/// use xrpl::types::{Amount, Asset, XChainBridge, transactions::xchain::XChainAddAccountCreateAttestation};
/// // Typically constructed by a witness server; fields are sourced from the source-chain event.
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct XChainAddAccountCreateAttestation {
    /// Amount committed on the source chain.
    pub amount: Amount,
    /// Account that receives the witness reward on the destination chain.
    pub attestation_reward_account: String,
    /// Account whose key signed the attestation.
    pub attestation_signer_account: String,
    /// Destination account to be created on the issuing chain.
    pub destination: String,
    /// Source account on the locking chain that submitted the commit.
    pub other_chain_source: String,
    /// Public key of the witness signer.
    #[serde(rename = "PublicKey")]
    pub public_key: String,
    /// Witness server's signature over the attestation data.
    pub signature: String,
    /// Reward paid to the witness for this attestation.
    pub signature_reward: Amount,
    /// `1` if the commit originated from the locking chain, `0` from the issuing chain.
    pub was_locking_chain_send: u8,
    /// Sequential counter for cross-chain account creation events (UInt64 as decimal string).
    #[serde(rename = "XChainAccountCreateCount")]
    pub xchain_account_create_count: String,
    /// Bridge configuration.
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge,
}

/// Witness server attestation that an `XChainCommit` occurred on the source chain.
///
/// Submitted by each witness server individually. Once a quorum of attestations is
/// collected for a given `xchain_claim_id`, the destination-chain `XChainClaim`
/// can succeed.
///
/// ```rust
/// use xrpl::types::{Amount, Asset, XChainBridge, transactions::xchain::XChainAddClaimAttestation};
/// // Typically constructed by a witness server; fields are sourced from the source-chain event.
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct XChainAddClaimAttestation {
    /// Amount committed on the source chain.
    pub amount: Amount,
    /// Account that receives the witness reward on the destination chain.
    pub attestation_reward_account: String,
    /// Account whose key signed the attestation.
    pub attestation_signer_account: String,
    /// Optional destination account override on the destination chain.
    pub destination: Option<String>,
    /// Source account on the origin chain that submitted the `XChainCommit`.
    pub other_chain_source: String,
    /// Public key of the witness signer.
    #[serde(rename = "PublicKey")]
    pub public_key: String,
    /// Witness server's signature over the attestation data.
    pub signature: String,
    /// `1` if the commit originated from the locking chain, `0` from the issuing chain.
    pub was_locking_chain_send: u8,
    /// Bridge configuration.
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge,
    /// Claim ID that corresponds to the `XChainCreateClaimID` on the destination chain.
    #[serde(rename = "XChainClaimID")]
    pub xchain_claim_id: String,
}

/// Completes a cross-chain transfer by claiming the committed assets on the destination chain.
///
/// Succeeds only after enough witness attestations have been submitted for the
/// associated `xchain_claim_id`.
///
/// ```rust
/// use xrpl::types::{Amount, Asset, XChainBridge, transactions::xchain::XChainClaim};
/// let tx = XChainClaim {
///     amount: Amount::Xrpl("100000000".to_string()),
///     destination: "rDestination".to_string(),
///     destination_tag: None,
///     xchain_bridge: XChainBridge {
///         locking_chain_door: "rLockDoor".to_string(),
///         locking_chain_issue: Asset::xrp(),
///         issuing_chain_door: "rIssueDoor".to_string(),
///         issuing_chain_issue: Asset::xrp(),
///     },
///     xchain_claim_id: "1".to_string(),
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct XChainClaim {
    /// Amount to receive on the destination chain.
    pub amount: Amount,
    /// Account on the destination chain that receives the assets.
    pub destination: String,
    /// Destination tag for routing within the destination account.
    pub destination_tag: Option<u32>,
    /// Bridge configuration.
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge,
    /// Claim ID created by `XChainCreateClaimID` on this chain.
    #[serde(rename = "XChainClaimID")]
    pub xchain_claim_id: String,
}

/// Locks or burns assets on the source chain to initiate a cross-chain transfer.
///
/// The `xchain_claim_id` must be obtained in advance via `XChainCreateClaimID` on
/// the destination chain. Witness servers observe this and submit attestations.
///
/// ```rust
/// use xrpl::types::{Amount, Asset, XChainBridge, transactions::xchain::XChainCommit};
/// let tx = XChainCommit {
///     amount: Amount::Xrpl("100000000".to_string()),
///     other_chain_destination: Some("rDestination".to_string()),
///     xchain_bridge: XChainBridge {
///         locking_chain_door: "rLockDoor".to_string(),
///         locking_chain_issue: Asset::xrp(),
///         issuing_chain_door: "rIssueDoor".to_string(),
///         issuing_chain_issue: Asset::xrp(),
///     },
///     xchain_claim_id: "1".to_string(),
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct XChainCommit {
    /// Amount of XRP or tokens to lock on the source chain.
    pub amount: Amount,
    /// Destination account on the other chain (overrides the one in `XChainClaim` if set).
    pub other_chain_destination: Option<String>,
    /// Bridge configuration.
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge,
    /// Claim ID obtained from `XChainCreateClaimID` on the destination chain.
    #[serde(rename = "XChainClaimID")]
    pub xchain_claim_id: String,
}

/// Registers a new cross-chain bridge on the ledger.
///
/// Must be submitted by the door account on both the locking chain and the issuing chain.
/// The `signature_reward` is distributed to witness servers for each attestation.
///
/// ```rust
/// use xrpl::types::{Amount, Asset, XChainBridge, transactions::xchain::XChainCreateBridge};
/// let tx = XChainCreateBridge {
///     signature_reward: Amount::Xrpl("100000000".to_string()),
///     min_account_create_amount: None,
///     xchain_bridge: XChainBridge {
///         locking_chain_door: "rLockDoor".to_string(),
///         locking_chain_issue: Asset::xrp(),
///         issuing_chain_door: "rIssueDoor".to_string(),
///         issuing_chain_issue: Asset::xrp(),
///     },
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct XChainCreateBridge {
    /// Total reward paid to witness servers per attestation batch.
    pub signature_reward: Amount,
    /// Minimum XRP required when creating an account on the issuing chain via this bridge.
    pub min_account_create_amount: Option<Amount>,
    /// Bridge configuration identifying the two chains and door accounts.
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge,
}

/// Reserves a cross-chain claim ID slot on the destination chain before a transfer begins.
///
/// The resulting claim ID must be included in the corresponding `XChainCommit` on the
/// source chain. One claim ID is consumed per cross-chain transfer.
///
/// ```rust
/// use xrpl::types::{Amount, Asset, XChainBridge, transactions::xchain::XChainCreateClaimID};
/// let tx = XChainCreateClaimID {
///     other_chain_source: "rSourceAccount".to_string(),
///     signature_reward: Amount::Xrpl("100000000".to_string()),
///     xchain_bridge: XChainBridge {
///         locking_chain_door: "rLockDoor".to_string(),
///         locking_chain_issue: Asset::xrp(),
///         issuing_chain_door: "rIssueDoor".to_string(),
///         issuing_chain_issue: Asset::xrp(),
///     },
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct XChainCreateClaimID {
    /// Account on the source chain that will submit the `XChainCommit`.
    pub other_chain_source: String,
    /// Reward amount paid to witness servers (must match the bridge's `SignatureReward`).
    pub signature_reward: Amount,
    /// Bridge configuration.
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge,
}

/// Updates parameters of an existing cross-chain bridge.
///
/// Only the door account that originally created the bridge can modify it.
/// At least one of `signature_reward` or `min_account_create_amount` must be provided.
///
/// ```rust
/// use xrpl::types::{Amount, Asset, XChainBridge, transactions::xchain::XChainModifyBridge};
/// let tx = XChainModifyBridge {
///     signature_reward: Some(Amount::Xrpl("200000000".to_string())),
///     min_account_create_amount: None,
///     xchain_bridge: XChainBridge {
///         locking_chain_door: "rLockDoor".to_string(),
///         locking_chain_issue: Asset::xrp(),
///         issuing_chain_door: "rIssueDoor".to_string(),
///         issuing_chain_issue: Asset::xrp(),
///     },
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct XChainModifyBridge {
    /// New total reward paid to witness servers per attestation batch.
    pub signature_reward: Option<Amount>,
    /// New minimum XRP required to create an account on the issuing chain via this bridge.
    pub min_account_create_amount: Option<Amount>,
    /// Bridge configuration identifying this bridge.
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge,
}
