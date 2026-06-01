use serde::{Deserialize, Serialize};
use super::{Amount, Asset};

/// One LP's vote for the AMM trading fee.
///
/// # Examples
///
/// ```rust
/// use xrpl::types::amm::VoteEntry;
/// // Returned as part of VoteSlots in an Amm ledger object.
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteEntry {
    /// r-address of the LP casting the vote.
    #[serde(rename = "Account")]
    pub account: String,
    /// Proposed trading fee in units of 1/100,000 (e.g. 500 = 0.5%).
    #[serde(rename = "TradingFee")]
    pub trading_fee: u16,
    /// Weight of this vote, proportional to the LP's token share.
    #[serde(rename = "VoteWeight")]
    pub vote_weight: u64,
}

/// Wire-format wrapper that nests a [`VoteEntry`] under the `VoteEntry` key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteSlotWrapper {
    /// The contained vote entry.
    #[serde(rename = "VoteEntry")]
    pub vote_entry: VoteEntry,
}

/// An account authorized to trade at a discounted fee during an auction slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthAccount {
    /// r-address of the authorized account.
    #[serde(rename = "Account")]
    pub account: String,
}

impl AuthAccount {
    /// Creates a new `AuthAccount` for the given r-address.
    pub fn new(account: impl Into<String>) -> Self {
        Self { account: account.into() }
    }
}

/// Wire-format wrapper that nests an [`AuthAccount`] under the `AuthAccount` key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthAccountWrapper {
    /// The contained authorized account.
    #[serde(rename = "AuthAccount")]
    pub auth_account: AuthAccount,
}

/// Alias kept for compatibility — the inner type of an auth-accounts list entry.
pub type AuthAccountEntry = AuthAccount;

/// The currently active auction slot of an AMM pool.
///
/// The auction-slot holder pays a discounted trading fee for the slot duration.
///
/// # Examples
///
/// ```rust
/// use xrpl::types::amm::AuctionSlot;
/// // Returned as part of an Amm ledger object when a slot is active.
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionSlot {
    /// r-address of the account that won the auction.
    #[serde(rename = "Account")]
    pub account: String,
    /// Additional accounts granted the discounted fee alongside the slot holder.
    #[serde(rename = "AuthAccounts")]
    pub auth_accounts: Option<Vec<AuthAccountEntry>>,
    /// Trading fee charged to the slot holder, in units of 1/100,000.
    #[serde(rename = "DiscountedFee")]
    pub discounted_fee: Option<u32>,
    /// Amount of LP tokens paid for the slot.
    #[serde(rename = "Price")]
    pub price: Amount,
    /// Ledger sequence at which the slot expires.
    #[serde(rename = "Expiration")]
    pub expiration: u32,
}

/// Ledger object representing an Automated Market Maker (AMM) pool.
///
/// Returned by the `amm_info` RPC command. Holds the full on-chain state of a
/// two-asset constant-product pool, including the current LP-token supply,
/// trading fee, and any active auction slot.
///
/// # Examples
///
/// ```rust
/// use xrpl::types::amm::Amm;
/// // Typically obtained via the amm_info WebSocket command response.
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Amm {
    /// Special AMM account that holds the pooled assets.
    #[serde(rename = "Account")]
    pub account: String,
    /// First asset in the pool.
    #[serde(rename = "Asset")]
    pub asset: Asset,
    /// Second asset in the pool.
    #[serde(rename = "Asset2")]
    pub asset2: Asset,
    /// Active auction slot, if any.
    #[serde(rename = "AuctionSlot")]
    pub auction_slot: Option<AuctionSlot>,
    /// Bitfield of AMM flags.
    #[serde(rename = "Flags")]
    pub flags: Option<u32>,
    /// Total outstanding LP token balance for this pool.
    #[serde(rename = "LPTokenBalance")]
    pub lp_token_balance: Amount,
    /// Ledger entry type discriminator (always `"AMM"`).
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: Option<String>,
    /// Index into the owner directory of the AMM account.
    #[serde(rename = "OwnerNode")]
    pub owner_node: Option<String>,
    /// Hash of the transaction that most recently modified this object.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Option<String>,
    /// Ledger sequence of the transaction that most recently modified this object.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: Option<u32>,
    /// Current pool trading fee in units of 1/100,000 (e.g. 500 = 0.5%).
    #[serde(rename = "TradingFee")]
    pub trading_fee: u16,
    /// Active fee-vote slots cast by LPs.
    #[serde(rename = "VoteSlots")]
    pub vote_slots: Option<Vec<VoteSlotWrapper>>,
    /// Ledger object index (hash).
    #[serde(rename = "index")]
    pub index: Option<String>,
}
