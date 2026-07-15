use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::types::{AccountFlag, SignerEntryWrapper};

/// Removes a funded account from the ledger, returning its remaining XRP to `Destination`.
///
/// The account's sequence number must be at least 256 ahead of its current ledger
/// sequence, and the transaction costs 2 XRP in addition to the normal fee.
///
/// ```rust
/// use xrpl::types::transactions::account::AccountDelete;
/// let tx = AccountDelete {
///     destination: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
///     destination_tag: Some(12345),
///     credential_ids: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccountDelete {
    /// Account that receives the remaining XRP balance.
    pub destination: String,
    /// Destination tag for routing within the destination account.
    pub destination_tag: Option<u32>,
    /// Credential IDs required to pass deposit authorization.
    #[serde(rename = "CredentialIDs")]
    pub credential_ids: Option<Vec<String>>,
}

/// Modifies account flags and optional properties such as domain, email hash, and transfer rate.
///
/// ```rust
/// use xrpl::types::transactions::account::AccountSet;
/// use xrpl::types::AccountFlag;
/// let tx = AccountSet {
///     set_flag: Some(AccountFlag::RequireDest),
///     domain: Some("6578616d706c652e636f6d".to_string()),
///     clear_flag: None,
///     email_hash: None,
///     message_key: None,
///     transfer_rate: None,
///     tick_size: None,
///     nftoken_minter: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccountSet {
    /// Account flag to disable.
    pub clear_flag: Option<AccountFlag>,
    /// Hex-encoded domain name associated with the account.
    pub domain: Option<String>,
    /// MD5 hash of an email address for Gravatar lookup.
    pub email_hash: Option<String>,
    /// Hex-encoded public key for encrypted messaging.
    pub message_key: Option<String>,
    /// Account flag to enable.
    pub set_flag: Option<AccountFlag>,
    /// Fee charged when users receive the issuer's tokens (in billionths, e.g. 1_005_000_000 = 0.5%).
    pub transfer_rate: Option<u32>,
    /// Minimum quote increment for offers on this account's issued currency (3-15, or 0 to disable).
    pub tick_size: Option<u32>,
    /// Account authorized to mint NFTokens on behalf of this account.
    #[serde(rename = "NFTokenMinter")]
    pub nftoken_minter: Option<String>,
}

/// Pre-authorizes or revokes a specific account's permission to send payments to this account.
///
/// Used when `DepositAuth` is enabled to explicitly whitelist senders without requiring
/// the sender to go through a separate authorization flow.
///
/// ```rust
/// use xrpl::types::transactions::account::DepositPreauth;
/// let tx = DepositPreauth {
///     authorize: Some("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string()),
///     unauthorize: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DepositPreauth {
    /// Account to grant deposit authorization to.
    pub authorize: Option<String>,
    /// Account whose deposit authorization is revoked.
    pub unauthorize: Option<String>,
}

/// Assigns or removes an alternate signing key pair for the account.
///
/// After setting a regular key, the account can be signed with either the master key
/// or the regular key. The master key can subsequently be disabled to improve security.
///
/// ```rust
/// use xrpl::types::transactions::account::SetRegularKey;
/// let tx = SetRegularKey {
///     regular_key: Some("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string()),
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SetRegularKey {
    /// The alternate signing key to assign; omit to remove the current regular key.
    pub regular_key: Option<String>,
}

/// Defines or replaces the multi-signature signer list and quorum for an account.
///
/// Submit with an empty `signer_entries` to delete the signer list and revert to
/// single-key signing.
///
/// ```rust
/// use xrpl::types::transactions::account::SignerListSet;
/// let tx = SignerListSet {
///     signer_quorum: 2,
///     signer_entries: None, // populated via builder
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SignerListSet {
    /// Minimum cumulative weight required to authorize a transaction.
    pub signer_quorum: u32,
    /// List of signer accounts and their individual weights.
    pub signer_entries: Option<Vec<SignerEntryWrapper>>,
}

/// Reserves one or more sequence-number slots (tickets) for out-of-order transaction submission.
///
/// Tickets allow sending transactions in an arbitrary order without being blocked by gaps
/// in the sequence number, which is useful for multi-signing workflows or parallel submissions.
///
/// ```rust
/// use xrpl::types::transactions::account::TicketCreate;
/// let tx = TicketCreate { ticket_count: 5 };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct TicketCreate {
    /// Number of tickets to reserve (1-250 per transaction).
    pub ticket_count: u32,
}
