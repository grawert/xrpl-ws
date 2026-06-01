use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};

/// Retrieves all open payment channels where the specified account is the source.
///
/// Use this to inspect how much XRP an account has allocated across its outbound
/// payment channels and what each channel's current balance is.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_channels::AccountChannelsRequest;
///
/// let req = AccountChannelsRequest {
///     account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///     limit: Some(50),
///     ..Default::default()
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct AccountChannelsRequest {
    /// Source account whose outbound channels are queried (r-address).
    pub account: String,
    /// Restrict results to channels whose destination is this account (r-address).
    pub destination_account: Option<String>,
    /// 64-hex-character hash of the ledger to query.
    pub ledger_hash: Option<String>,
    /// Ledger to query: a sequence number, or a shortcut such as `"validated"`.
    pub ledger_index: Option<Value>,
    /// Maximum number of channels to return in a single response.
    pub limit: Option<u32>,
    /// Pagination cursor returned by a previous response; pass back to fetch the next page.
    pub marker: Option<Value>,
}

impl XrplRequest for AccountChannelsRequest {
    type Response = XrplResponse<AccountChannelsResponse>;
    const COMMAND: &str = "account_channels";
}

/// Response payload for an [`AccountChannelsRequest`].
///
/// Contains the list of open payment channels owned by the queried account
/// along with ledger context and pagination state.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_channels::AccountChannelsResponse;
///
/// fn print_totals(resp: &AccountChannelsResponse) {
///     for ch in &resp.channels {
///         println!("channel {} — allocated: {} drops", ch.channel_id, ch.amount);
///     }
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct AccountChannelsResponse {
    /// Source account address (r-address) whose channels are returned.
    pub account: String,
    /// List of open payment channels sourced from `account`.
    pub channels: Vec<AccountChannel>,
    /// Hash of the ledger used to answer the request.
    pub ledger_hash: Option<String>,
    /// Sequence number of the ledger used to answer the request.
    pub ledger_index: Option<u32>,
    /// `true` when the response is based on a validated (immutable) ledger.
    pub validated: Option<bool>,
    /// Pagination cursor; present when more channels remain on the next page.
    pub marker: Option<Value>,
    /// Effective page size applied by the server.
    pub limit: Option<u32>,
}

/// A single unidirectional XRP payment channel.
///
/// Represents a channel opened by the source account that allows off-ledger
/// micro-payments to the destination, settling on-ledger via claims.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_channels::AccountChannel;
///
/// fn available_drops(ch: &AccountChannel) -> u64 {
///     ch.amount.parse::<u64>().unwrap_or(0)
///         .saturating_sub(ch.balance.parse::<u64>().unwrap_or(0))
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct AccountChannel {
    /// Source account address (r-address) that funded the channel.
    pub account: String,
    /// Total XRP (in drops) allocated to the channel.
    pub amount: String,
    /// XRP (in drops) already claimed by the destination.
    pub balance: String,
    /// Unique 64-hex-character channel identifier.
    pub channel_id: String,
    /// Destination account that can claim XRP from the channel (r-address).
    pub destination_account: String,
    /// Minimum seconds the source must wait to close the channel after requesting closure.
    pub settle_delay: u32,
    /// Source's secp256k1 public key for signing channel claims (base58).
    pub public_key: Option<String>,
    /// Source's public key in hex format.
    pub public_key_hex: Option<String>,
    /// Ripple epoch timestamp when the channel expires (mutable, set by source).
    pub expiration: Option<u32>,
    /// Ripple epoch timestamp after which anyone can close the channel (immutable).
    pub cancel_after: Option<u32>,
    /// Source-defined tag for routing or reference.
    pub source_tag: Option<u32>,
    /// Destination-defined tag for routing or reference.
    pub destination_tag: Option<u32>,
}
