use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};

/// Retrieves NFTokens owned by an account (XLS-20).
///
/// Returns metadata for each NFToken the account currently holds, including
/// the token ID, issuer, taxon, and URI. Paginate with `limit` and `marker`
/// for accounts with large NFT collections.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_nfts::AccountNftsRequest;
///
/// let req = AccountNftsRequest {
///     account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///     limit: Some(100),
///     ..Default::default()
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct AccountNftsRequest {
    /// Account whose NFTokens are queried (r-address).
    pub account: String,
    /// 64-hex-character hash of the ledger to query.
    pub ledger_hash: Option<String>,
    /// Ledger to query: a sequence number, or a shortcut such as `"validated"`.
    pub ledger_index: Option<Value>,
    /// Maximum number of NFTokens to return in a single response.
    pub limit: Option<u32>,
    /// Pagination cursor returned by a previous response; pass back to fetch the next page.
    pub marker: Option<Value>,
}

impl XrplRequest for AccountNftsRequest {
    type Response = XrplResponse<AccountNftsResponse>;
    const COMMAND: &str = "account_nfts";
}

/// Response payload for an [`AccountNftsRequest`].
///
/// Contains the page of NFTokens owned by the queried account along with ledger
/// context and a pagination marker for retrieving subsequent pages.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_nfts::AccountNftsResponse;
///
/// fn count_nfts(resp: &AccountNftsResponse) -> usize {
///     resp.account_nfts.len()
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct AccountNftsResponse {
    /// Account whose NFTokens are returned (r-address).
    pub account: String,
    /// NFTokens currently owned by the account.
    pub account_nfts: Vec<AccountNFToken>,
    /// Hash of the ledger used to answer the request.
    pub ledger_hash: Option<String>,
    /// Sequence number of the validated ledger used to answer the request.
    pub ledger_index: Option<u32>,
    /// Sequence number of the current open ledger (present when querying the open ledger).
    pub ledger_current_index: Option<u32>,
    /// `true` when the response is based on a validated (immutable) ledger.
    pub validated: Option<bool>,
    /// Pagination cursor; present when more NFTokens remain on the next page.
    pub marker: Option<Value>,
    /// Effective page size applied by the server.
    pub limit: Option<u32>,
}

/// A single NFToken owned by an account (XLS-20).
///
/// Carries the immutable metadata of the token as recorded in the ledger.
/// Wire fields are PascalCase (`Flags`, `Issuer`, `NFTokenID`, …).
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_nfts::AccountNFToken;
///
/// fn is_transferable(token: &AccountNFToken) -> bool {
///     // tfTransferable flag bit
///     token.flags & 0x0008 != 0
/// }
/// ```
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccountNFToken {
    /// Bitfield of NFToken flags (e.g. `tfTransferable`, `tfOnlyXRP`). Wire: `Flags`.
    pub flags: u32,
    /// Account that minted the token (r-address). Wire: `Issuer`.
    pub issuer: String,
    /// Unique 256-bit token identifier (64 hex chars). Wire: `NFTokenID`.
    #[serde(rename = "NFTokenID")]
    pub nftoken_id: String,
    /// Issuer-defined taxon that groups related tokens. Wire: `NFTokenTaxon`.
    #[serde(rename = "NFTokenTaxon")]
    pub nftoken_taxon: u32,
    /// Hex-encoded URI pointing to the token's metadata (e.g. IPFS). Wire: `URI`.
    #[serde(rename = "URI")]
    pub uri: Option<String>,
    /// Per-issuer-taxon serial number assigned at mint time. Wire: `nft_serial`.
    #[serde(rename = "nft_serial")]
    pub nft_serial: u32,
}
