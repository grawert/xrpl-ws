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
/// let request = LedgerEntryRequest::by_index("7DB0788C020F02780A673DC74757F23823FA3014C1866E72CC4CD8B226CD6EF4")
///     .with_ledger_index("validated");
/// ```
///
/// Look up an account's root object:
/// ```rust
/// use xrpl::request::ledger_entry::LedgerEntryRequest;
/// let request = LedgerEntryRequest::for_account_root("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
pub struct LedgerEntryRequest {
    /// Ledger hash to target a specific ledger version.
    pub ledger_hash: Option<String>,
    /// Ledger index or shortcut ("validated", "closed", "current").
    pub ledger_index: Option<Value>,
    /// If true, return the entry as a binary blob instead of JSON.
    pub binary: Option<bool>,
    /// (Clio only) Return the complete data as it was prior to its deletion if the queried object has been deleted.
    pub include_deleted: Option<bool>,

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

impl LedgerEntryRequest {
    /// Creates a request to look up an entry by its 64-character hex index.
    pub fn by_index(index: impl AsRef<str>) -> Self {
        Self { index: Some(index.as_ref().to_string()), ..Default::default() }
    }

    /// Creates a request for an AccountRoot entry.
    pub fn for_account_root(account: impl AsRef<str>) -> Self {
        Self {
            account_root: Some(account.as_ref().to_string()),
            ..Default::default()
        }
    }

    /// Creates a request for an Escrow entry identified by owner and sequence number.
    pub fn for_escrow(owner: impl AsRef<str>, seq: u32) -> Self {
        Self {
            escrow: Some(EscrowLedgerKey::new(owner, seq)),
            ..Default::default()
        }
    }

    /// Creates a request for an Offer entry identified by account and sequence number.
    pub fn for_offer(account: impl AsRef<str>, seq: u32) -> Self {
        Self {
            offer: Some(OfferLedgerKey::new(account, seq)),
            ..Default::default()
        }
    }

    /// Creates a request for a trust line (RippleState) entry identified by two accounts and a currency code.
    pub fn for_ripple_state(
        accounts: [impl AsRef<str>; 2],
        currency: impl AsRef<str>,
    ) -> Self {
        let [a, b] = accounts;
        Self {
            ripple_state: Some(RippleStateLedgerKey::new(a, b, currency)),
            ..Default::default()
        }
    }

    /// Creates a request for an AMM pool entry identified by its two assets.
    pub fn for_amm(asset: impl Into<Asset>, asset2: impl Into<Asset>) -> Self {
        Self {
            amm: Some(AmmLedgerKey::new(asset, asset2)),
            ..Default::default()
        }
    }

    /// Creates a request for a Check entry.
    pub fn for_check(id: impl AsRef<str>) -> Self {
        Self { check: Some(id.as_ref().to_string()), ..Default::default() }
    }

    /// Creates a request for an NFTokenPage entry.
    pub fn for_nft_page(id: impl AsRef<str>) -> Self {
        Self { nft_page: Some(id.as_ref().to_string()), ..Default::default() }
    }

    /// Creates a request for an NFTokenOffer entry.
    pub fn for_nft_offer(id: impl AsRef<str>) -> Self {
        Self { nft_offer: Some(id.as_ref().to_string()), ..Default::default() }
    }

    /// Creates a request for a PaymentChannel entry.
    pub fn for_payment_channel(id: impl AsRef<str>) -> Self {
        Self {
            payment_channel: Some(id.as_ref().to_string()),
            ..Default::default()
        }
    }

    /// Creates a request for a DID entry.
    pub fn for_did(account: impl AsRef<str>) -> Self {
        Self { did: Some(account.as_ref().to_string()), ..Default::default() }
    }

    /// Creates a request for an MPTokenIssuance entry.
    pub fn for_mpt_issuance(id: impl AsRef<str>) -> Self {
        Self {
            mpt_issuance: Some(id.as_ref().to_string()),
            ..Default::default()
        }
    }

    /// Creates a request for a SignerList entry.
    pub fn for_signer_list(id: impl AsRef<str>) -> Self {
        Self {
            signer_list: Some(id.as_ref().to_string()),
            ..Default::default()
        }
    }

    /// Creates a request for a Vault entry.
    pub fn for_vault(id: impl AsRef<str>) -> Self {
        Self { vault: Some(id.as_ref().to_string()), ..Default::default() }
    }

    /// Creates a request for a Ticket entry identified by account and ticket sequence number.
    pub fn for_ticket(account: impl AsRef<str>, ticket_seq: u32) -> Self {
        Self {
            ticket: Some(TicketLedgerKey::new(account, ticket_seq)),
            ..Default::default()
        }
    }

    /// Creates a request for a DepositPreauth entry identified by owner and authorized account.
    pub fn for_deposit_preauth(
        owner: impl AsRef<str>,
        authorized: impl AsRef<str>,
    ) -> Self {
        Self {
            deposit_preauth: Some(DepositPreauthLedgerKey::new(
                owner, authorized,
            )),
            ..Default::default()
        }
    }

    /// Creates a request for the Amendments singleton entry.
    pub fn for_amendments() -> Self {
        Self { amendments: Some(String::new()), ..Default::default() }
    }

    /// Creates a request for the FeeSettings singleton entry.
    pub fn for_fee_settings() -> Self {
        Self { fee: Some(String::new()), ..Default::default() }
    }

    /// Creates a request for the LedgerHashes singleton entry.
    pub fn for_hashes() -> Self {
        Self { hashes: Some(String::new()), ..Default::default() }
    }

    /// Creates a request for the NegativeUNL singleton entry.
    pub fn for_nunl() -> Self {
        Self { nunl: Some(String::new()), ..Default::default() }
    }

    /// Sets the target ledger hash.
    pub fn with_ledger_hash(mut self, hash: impl AsRef<str>) -> Self {
        self.ledger_hash = Some(hash.as_ref().to_string());
        self
    }

    /// Sets the ledger index or shortcut.
    pub fn with_ledger_index(mut self, index: impl Into<Value>) -> Self {
        self.ledger_index = Some(index.into());
        self
    }

    /// Configures whether to return binary data instead of JSON.
    pub fn with_binary(mut self, binary: bool) -> Self {
        self.binary = Some(binary);
        self
    }

    /// Configures whether to include deleted objects.
    pub fn with_include_deleted(mut self, include_deleted: bool) -> Self {
        self.include_deleted = Some(include_deleted);
        self
    }
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
#[derive(Debug, Clone, Serialize)]
pub struct EscrowLedgerKey {
    /// Account that created the escrow.
    pub owner: String,
    /// Sequence number of the EscrowCreate transaction.
    pub seq: u32,
}

impl EscrowLedgerKey {
    /// Creates a new key for Escrow lookup.
    pub fn new(owner: impl AsRef<str>, seq: u32) -> Self {
        Self { owner: owner.as_ref().to_string(), seq }
    }
}

/// Key for looking up an Offer entry: `{account, seq}`.
#[derive(Debug, Clone, Serialize)]
pub struct OfferLedgerKey {
    /// Account that placed the offer.
    pub account: String,
    /// Sequence number of the OfferCreate transaction.
    pub seq: u32,
}

impl OfferLedgerKey {
    /// Creates a new key for Offer lookup.
    pub fn new(account: impl AsRef<str>, seq: u32) -> Self {
        Self { account: account.as_ref().to_string(), seq }
    }
}

/// Key for looking up a trust line (RippleState): `{accounts: [A, B], currency}`.
#[derive(Debug, Clone, Serialize)]
pub struct RippleStateLedgerKey {
    /// The two accounts sharing this trust line (order does not matter).
    pub accounts: [String; 2],
    /// ISO 4217 currency code or 40-character hex non-standard currency code.
    pub currency: String,
}

impl RippleStateLedgerKey {
    /// Creates a new key for RippleState lookup.
    pub fn new(
        account1: impl AsRef<str>,
        account2: impl AsRef<str>,
        currency: impl AsRef<str>,
    ) -> Self {
        Self {
            accounts: [
                account1.as_ref().to_string(),
                account2.as_ref().to_string(),
            ],
            currency: currency.as_ref().to_string(),
        }
    }
}

/// Key for looking up a Ticket entry: `{account, ticket_seq}`.
#[derive(Debug, Clone, Serialize)]
pub struct TicketLedgerKey {
    /// Account that created the ticket.
    pub account: String,
    /// Sequence number reserved by the ticket.
    pub ticket_seq: u32,
}

impl TicketLedgerKey {
    /// Creates a new key for Ticket lookup.
    pub fn new(account: impl AsRef<str>, ticket_seq: u32) -> Self {
        Self { account: account.as_ref().to_string(), ticket_seq }
    }
}

/// Key for looking up a DepositPreauth entry: `{owner, authorized}`.
#[derive(Debug, Clone, Serialize)]
pub struct DepositPreauthLedgerKey {
    /// Account that granted the preauthorization.
    pub owner: String,
    /// Account that was preauthorized to send payments.
    pub authorized: String,
}

impl DepositPreauthLedgerKey {
    /// Creates a new key for DepositPreauth lookup.
    pub fn new(owner: impl AsRef<str>, authorized: impl AsRef<str>) -> Self {
        Self {
            owner: owner.as_ref().to_string(),
            authorized: authorized.as_ref().to_string(),
        }
    }
}

/// Key for looking up an AMM entry by its two assets.
#[derive(Debug, Clone, Serialize)]
pub struct AmmLedgerKey {
    /// First asset in the AMM pair.
    pub asset: Asset,
    /// Second asset in the AMM pair.
    pub asset2: Asset,
}

impl AmmLedgerKey {
    /// Creates a new key for AMM lookup.
    pub fn new(asset: impl Into<Asset>, asset2: impl Into<Asset>) -> Self {
        Self { asset: asset.into(), asset2: asset2.into() }
    }
}
