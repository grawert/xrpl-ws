use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};
use crate::types::AccountObject;

/// Retrieves all ledger objects owned by an account.
///
/// Covers any object type that counts against the account's owner reserve: offers,
/// escrows, payment channels, trust lines, signer lists, tickets, checks, and more.
/// Filter by type with `kind`, or set `deletion_blockers_only` to find objects that
/// prevent account deletion.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_objects::{AccountObjectsRequest, AccountObjectType};
///
/// let req = AccountObjectsRequest {
///     account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///     kind: Some(AccountObjectType::Offer),
///     limit: Some(100),
///     ..Default::default()
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct AccountObjectsRequest {
    /// Account whose owned objects are queried (r-address).
    pub account: String,
    /// When `true`, return only objects that block account deletion.
    pub deletion_blockers_only: Option<bool>,
    /// 64-hex-character hash of the ledger to query.
    pub ledger_hash: Option<String>,
    /// Ledger to query: a sequence number, or a shortcut such as `"validated"`.
    pub ledger_index: Option<Value>,
    /// Maximum number of objects to return in a single response.
    pub limit: Option<u32>,
    /// Pagination cursor returned by a previous response; pass back to fetch the next page.
    pub marker: Option<Value>,
    /// Filter results to include only objects of this specific type.
    #[serde(rename = "type")]
    pub kind: Option<AccountObjectType>,
}

impl AccountObjectsRequest {
    /// Creates a new request for the given account address.
    ///
    /// # Example
    /// ```rust
    /// use xrpl::request::account_objects::AccountObjectsRequest;
    /// let req = AccountObjectsRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    /// ```
    pub fn new(account: impl Into<String>) -> Self {
        Self { account: account.into(), ..Default::default() }
    }
}

impl XrplRequest for AccountObjectsRequest {
    type Response = XrplResponse<AccountObjectsResponse>;
    const COMMAND: &str = "account_objects";
}

/// Filter for [`AccountObjectsRequest`] restricting results to one ledger object type.
///
/// Wire values are snake_case strings (e.g. `"offer"`, `"payment_channel"`).
/// Use this to narrow a query to only the object category you care about, reducing
/// the result set and avoiding unnecessary pagination.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_objects::{AccountObjectsRequest, AccountObjectType};
///
/// let req = AccountObjectsRequest {
///     account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///     kind: Some(AccountObjectType::Escrow),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountObjectType {
    /// Cross-chain bridge object.
    Bridge,
    /// Check object (deferred payment authorization).
    Check,
    /// Credential object.
    Credential,
    /// Delegate object.
    Delegate,
    /// Deposit pre-authorization object.
    DepositPreauth,
    /// DID (Decentralized Identifier) object. Wire: `"did"`.
    #[serde(rename = "did")]
    DID,
    /// Escrow object holding conditional or time-locked XRP.
    Escrow,
    /// Multi-Purpose Token (MPT) holding. Wire: `"mptoken"`.
    #[serde(rename = "mptoken")]
    MPToken,
    /// Multi-Purpose Token issuance object. Wire: `"mpt_issuance"`.
    #[serde(rename = "mpt_issuance")]
    MPTokenIssuance,
    /// NFToken buy or sell offer. Wire: `"nft_offer"`.
    #[serde(rename = "nft_offer")]
    NFTokenOffer,
    /// NFToken page storing up to 32 NFTokens. Wire: `"nft_page"`.
    #[serde(rename = "nft_page")]
    NFTokenPage,
    /// DEX limit order placed by the account.
    Offer,
    /// Oracle object.
    Oracle,
    /// Unidirectional XRP payment channel. Wire: `"payment_channel"`.
    #[serde(rename = "payment_channel")]
    PayChannel,
    /// Permissioned domain object.
    PermissionedDomain,
    /// Trust line (RippleState) between two accounts. Wire: `"state"`.
    #[serde(rename = "state")]
    RippleState,
    /// Multi-signature signer list attached to the account.
    SignerList,
    /// Ticket reserving a future sequence number.
    Ticket,
    /// XChain owned claim ID. Wire: `"xchain_owned_claim_id"`.
    #[serde(rename = "xchain_owned_claim_id")]
    XChainOwnedClaimId,
    /// XChain owned create account claim ID. Wire: `"xchain_owned_create_account_claim_id"`.
    #[serde(rename = "xchain_owned_create_account_claim_id")]
    XChainOwnedCreateAccountClaimId,
}

/// Response payload for an [`AccountObjectsRequest`].
///
/// Contains the page of ledger objects owned by the queried account along with
/// ledger context and a pagination marker for retrieving subsequent pages.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::account_objects::AccountObjectsResponse;
///
/// fn object_count(resp: &AccountObjectsResponse) -> usize {
///     resp.account_objects.len()
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct AccountObjectsResponse {
    /// Account whose objects are returned (r-address).
    pub account: String,
    /// Ledger objects owned by the account in the queried ledger.
    pub account_objects: Vec<AccountObject>,
    /// Hash of the ledger used to answer the request.
    pub ledger_hash: Option<String>,
    /// Sequence number of the validated ledger used to answer the request.
    pub ledger_index: Option<u32>,
    /// Sequence number of the current open ledger (present when querying the open ledger).
    pub ledger_current_index: Option<u32>,
    /// Effective page size applied by the server.
    pub limit: Option<u32>,
    /// Pagination cursor; present when more objects remain on the next page.
    pub marker: Option<Value>,
    /// `true` when the response is based on a validated (immutable) ledger.
    pub validated: Option<bool>,
}
