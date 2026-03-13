use serde::{Deserialize, Serialize};
use super::Amount;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteEntry {
    #[serde(rename = "Account")]
    pub account: String,
    #[serde(rename = "TradingFee")]
    pub trading_fee: u16,
    #[serde(rename = "VoteWeight")]
    pub vote_weight: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteSlotWrapper {
    #[serde(rename = "VoteEntry")]
    pub vote_entry: VoteEntry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthAccount {
    #[serde(rename = "Account")]
    pub account: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthAccountWrapper {
    #[serde(rename = "AuthAccount")]
    pub auth_account: AuthAccount,
}

// Alias for backwards compatibility with ledger entry format
pub type AuthAccountEntry = AuthAccount;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionSlot {
    #[serde(rename = "Account")]
    pub account: String,
    #[serde(rename = "AuthAccounts")]
    pub auth_accounts: Option<Vec<AuthAccountEntry>>,
    #[serde(rename = "DiscountedFee")]
    pub discounted_fee: Option<u32>,
    #[serde(rename = "Price")]
    pub price: Amount,
    #[serde(rename = "Expiration")]
    pub expiration: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Amm {
    #[serde(rename = "Account")]
    pub account: String,
    #[serde(rename = "Asset")]
    pub asset: Amount,
    #[serde(rename = "Asset2")]
    pub asset2: Amount,
    #[serde(rename = "AuctionSlot")]
    pub auction_slot: Option<AuctionSlot>,
    #[serde(rename = "Flags")]
    pub flags: Option<u32>,
    #[serde(rename = "LPTokenBalance")]
    pub lp_token_balance: Amount,
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: Option<String>,
    #[serde(rename = "OwnerNode")]
    pub owner_node: Option<String>,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Option<String>,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: Option<u32>,
    #[serde(rename = "TradingFee")]
    pub trading_fee: u16,
    #[serde(rename = "VoteSlots")]
    pub vote_slots: Option<Vec<VoteSlotWrapper>>,
    #[serde(rename = "index")]
    pub index: Option<String>,
}
