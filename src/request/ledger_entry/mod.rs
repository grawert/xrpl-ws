use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};
use crate::types::Asset;

/// Retrieves a single ledger entry by its identifying key.
///
/// Set exactly one of the key fields. All other key fields must be `None`.
///
/// # Examples
///
/// Look up a ledger entry directly by its index:
/// ```rust
/// use xrpl::request::ledger_entry::LedgerEntryRequest;
///
/// let request = LedgerEntryRequest {
///     index: Some("7DB0788C020F02780A673DC74757F23823FA3014C1866E72CC4CD8B226CD6EF4".to_string()),
///     ledger_index: Some("validated".into()),
///     ..Default::default()
/// };
/// ```
///
/// Look up an account's root object:
/// ```rust
/// use xrpl::request::ledger_entry::LedgerEntryRequest;
///
/// let request = LedgerEntryRequest {
///     account_root: Some("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string()),
///     ..Default::default()
/// };
/// ```
///
/// Look up a trust line between two accounts:
/// ```rust
/// use xrpl::request::ledger_entry::{LedgerEntryRequest, RippleStateLedgerKey};
///
/// let request = LedgerEntryRequest {
///     ripple_state: Some(RippleStateLedgerKey {
///         accounts: ["rA...".to_string(), "rB...".to_string()],
///         currency: "USD".to_string(),
///     }),
///     ..Default::default()
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct LedgerEntryRequest {
    /// Ledger hash to target a specific ledger version.
    pub ledger_hash: Option<String>,
    /// Ledger index or shortcut ("validated", "closed", "current").
    pub ledger_index: Option<Value>,
    /// If true, return the entry as a binary blob instead of JSON.
    pub binary: Option<bool>,
    /// (Clio only) Return the complete data as it was prior to its deletion if the queried object has been deleted.
    pub include_deleted: Option<bool>,

    // --- Simple (string) key lookups ---
    /// Direct lookup by the 64-character hex ledger entry index.
    pub index: Option<String>,
    /// Account address for an AccountRoot entry.
    pub account_root: Option<String>,
    /// The Amendments entry.
    pub amendments: Option<String>,
    /// Check ID (64-character hex).
    pub check: Option<String>,
    /// The FeeSettings entry.
    pub fee: Option<String>,
    /// The LedgerHashes entry.
    pub hashes: Option<String>,
    /// The NegativeUNL entry.
    pub nunl: Option<String>,
    /// The NFT Page ID.
    pub nft_page: Option<String>,
    /// NFToken offer ID.
    pub nft_offer: Option<String>,
    /// Payment channel ID.
    pub payment_channel: Option<String>,
    /// Account address for a DID entry.
    pub did: Option<String>,
    /// The MPTokenIssuance ID.
    pub mpt_issuance: Option<String>,
    /// The SignerList ID.
    pub signer_list: Option<String>,
    /// The Vault ID.
    pub vault: Option<String>,

    // --- Compound key lookups ---
    /// Key for an Escrow entry.
    pub escrow: Option<EscrowLedgerKey>,
    /// Key for an Offer entry.
    pub offer: Option<OfferLedgerKey>,
    /// Key for a trust line (RippleState) entry.
    pub ripple_state: Option<RippleStateLedgerKey>,
    /// Key for a Ticket entry.
    pub ticket: Option<TicketLedgerKey>,
    /// Key for a DepositPreauth entry.
    pub deposit_preauth: Option<DepositPreauthLedgerKey>,
    /// Key for an AMM pool entry.
    pub amm: Option<AmmLedgerKey>,

    // --- Less common (raw Value) ---
    /// Key for a DirectoryNode entry.
    pub directory: Option<Value>,
    /// Key for an XChainBridge entry.
    pub bridge: Option<Value>,
    /// Key for a PriceOracle entry.
    pub oracle: Option<Value>,
    /// Key for a Credential entry.
    pub credential: Option<Value>,
    /// Key for an XChainOwnedClaimID entry.
    pub xchain_owned_claim_id: Option<Value>,
    /// Key for an XChainOwnedCreateAccountClaimID entry.
    pub xchain_owned_create_account_claim_id: Option<Value>,
    /// Key for a Loan entry.
    pub loan: Option<Value>,
    /// Key for a LoanBroker entry.
    pub loan_broker: Option<Value>,
    /// Key for an MPToken entry.
    pub mptoken: Option<Value>,
    /// Key for a PermissionedDomain entry.
    pub permissioned_domain: Option<Value>,
}

impl XrplRequest for LedgerEntryRequest {
    type Response = XrplResponse<LedgerEntryResponse>;
    const COMMAND: &str = "ledger_entry";
}

/// Response to a `ledger_entry` request.
#[derive(Debug, Deserialize)]
pub struct LedgerEntryResponse {
    /// The 64-character hex index of the ledger entry.
    pub index: String,
    /// Sequence number of the current open ledger (unvalidated results).
    pub ledger_current_index: Option<u32>,
    /// Sequence number of the ledger version used.
    pub ledger_index: Option<u32>,
    /// Hash of the ledger version used.
    pub ledger_hash: Option<String>,
    /// The ledger entry in JSON format. `None` when `binary` is `true`.
    pub node: Option<Value>,
    /// The ledger entry in binary format. `None` when `binary` is `false`.
    pub node_binary: Option<String>,
    /// Whether the data comes from a validated ledger.
    pub validated: Option<bool>,
    /// (Clio server only) The ledger index where the ledger entry object was deleted.
    pub deleted_ledger_index: Option<String>,
}

/// Key for looking up an Escrow entry: `{owner, seq}`.
#[derive(Debug, Serialize)]
pub struct EscrowLedgerKey {
    /// Account that created the escrow.
    pub owner: String,
    /// Sequence number of the EscrowCreate transaction.
    pub seq: u32,
}

/// Key for looking up an Offer entry: `{account, seq}`.
#[derive(Debug, Serialize)]
pub struct OfferLedgerKey {
    /// Account that placed the offer.
    pub account: String,
    /// Sequence number of the OfferCreate transaction.
    pub seq: u32,
}

/// Key for looking up a trust line (RippleState): `{accounts: [A, B], currency}`.
#[derive(Debug, Serialize)]
pub struct RippleStateLedgerKey {
    /// The two accounts sharing this trust line (order does not matter).
    pub accounts: [String; 2],
    /// ISO 4217 currency code or 40-character hex non-standard currency code.
    pub currency: String,
}

/// Key for looking up a Ticket entry: `{account, ticket_seq}`.
#[derive(Debug, Serialize)]
pub struct TicketLedgerKey {
    /// Account that created the ticket.
    pub account: String,
    /// Sequence number reserved by the ticket.
    pub ticket_seq: u32,
}

/// Key for looking up a DepositPreauth entry: `{owner, authorized}`.
#[derive(Debug, Serialize)]
pub struct DepositPreauthLedgerKey {
    /// Account that granted the preauthorization.
    pub owner: String,
    /// Account that was preauthorized to send payments.
    pub authorized: String,
}

/// Key for looking up an AMM entry by its two assets.
#[derive(Debug, Serialize)]
pub struct AmmLedgerKey {
    /// First asset in the AMM pair.
    pub asset: Asset,
    /// Second asset in the AMM pair.
    pub asset2: Asset,
}
