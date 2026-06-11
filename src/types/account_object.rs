use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use super::{Amount, XChainBridge};

/// Any ledger object that an account can own, discriminated by `LedgerEntryType`.
///
/// Returned inside the `account_objects` array of the `account_objects` RPC command.
/// Match on this enum to access type-specific fields without casting.
///
/// # Examples
///
/// ```rust
/// use xrpl::types::account_object::AccountObject;
/// // Typically deserialized from the account_objects RPC response.
/// ```
#[derive(Debug, Deserialize)]
#[serde(tag = "LedgerEntryType")]
pub enum AccountObject {
    /// A cross-chain bridge door entry owned by this account.
    Bridge(Bridge),
    /// A deferred payment check that can be cashed by the destination.
    Check(Check),
    /// A verifiable credential issued to or by this account.
    Credential(Credential),
    /// A deposit pre-authorization granted by this account.
    DepositPreauth(DepositPreauth),
    /// A Decentralized Identifier (DID) document anchored to this account.
    #[serde(rename = "DID")]
    Did(Did),
    /// A time-locked or condition-locked XRP escrow.
    Escrow(Escrow),
    /// A Multi-Purpose Token holding owned by this account.
    MPToken(MPToken),
    /// An MPT issuance created by this account.
    MPTokenIssuance(MPTokenIssuance),
    /// An offer to buy or sell an NFToken.
    NFTokenOffer(NFTokenOffer),
    /// A page of NFTokens stored in this account's collection.
    NFTokenPage(NFTokenPage),
    /// A DEX offer to exchange one asset for another.
    Offer(Offer),
    /// A price oracle entry published by this account.
    Oracle(Oracle),
    /// A payment channel funded by this account.
    PayChannel(PayChannel),
    /// A trust line (RippleState) between this account and a counterparty.
    RippleState(RippleState),
    /// A multi-signature signer list associated with this account.
    SignerList(SignerList),
    /// A sequence-number ticket reserved for a future transaction.
    Ticket(Ticket),
    /// A cross-chain claim ID owned by this account.
    XChainOwnedClaimID(XChainOwnedClaimID),
    /// A cross-chain create-account claim ID owned by this account.
    XChainOwnedCreateAccountClaimID(XChainOwnedCreateAccountClaimID),
}

/// Fields present on every ledger object; flattened into each concrete type.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Common {
    /// Bitfield of object-specific flags.
    pub flags: u32,
    /// Ledger object index (hash), when included in responses.
    pub index: Option<String>,
    /// Index into the owner directory page that holds this object.
    pub owner_node: Option<String>,
    /// Hash of the transaction that most recently modified this object.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Option<String>,
    /// Ledger sequence of the transaction that most recently modified this object.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: Option<u32>,
}

/// A deferred payment check that the destination can cash for up to `send_max`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Check {
    /// r-address of the account that created the check.
    pub account: String,
    /// r-address of the account authorized to cash the check.
    pub destination: String,
    /// Index into the destination's owner directory.
    pub destination_node: Option<String>,
    /// Destination tag for routing within the destination account.
    pub destination_tag: Option<u32>,
    /// Ripple epoch time after which the check can no longer be cashed.
    pub expiration: Option<u32>,
    /// Optional 256-bit hash identifying the invoice this check is for.
    #[serde(rename = "InvoiceID")]
    pub invoice_id: Option<String>,
    /// Maximum amount the destination can receive when cashing.
    pub send_max: Value,
    /// Sequence number of the CheckCreate transaction.
    pub sequence: u32,
    /// Source tag for routing within the sending account.
    pub source_tag: Option<u32>,

    /// Shared ledger-object metadata (flags, index, previous transaction reference).
    #[serde(flatten)]
    pub common: Common,
}

/// A verifiable credential issued to `account` by `issuer`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Credential {
    /// r-address of the credential subject (holder).
    pub account: String,
    /// r-address of the credential issuer.
    pub issuer: String,
    /// Hex-encoded credential type identifier.
    pub credential_type: String,
    /// Ripple epoch time after which the credential expires.
    pub expiration: Option<u32>,
    /// Optional URI pointing to additional credential metadata.
    #[serde(rename = "URI")]
    pub uri: Option<String>,

    /// Shared ledger-object metadata (flags, index, previous transaction reference).
    #[serde(flatten)]
    pub common: Common,
}

/// A deposit pre-authorization allowing a specific sender to make payments.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DepositPreauth {
    /// r-address of the account that granted the pre-authorization.
    pub account: String,
    /// r-address of the account granted permission to send deposits.
    pub authorize: Option<String>,
    /// Credential-based authorization entries (XLS-34).
    pub authorize_credentials: Option<Value>,

    /// Shared ledger-object metadata (flags, index, previous transaction reference).
    #[serde(flatten)]
    pub common: Common,
}

/// A Decentralized Identifier (DID) document anchored on the XRPL.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Did {
    /// r-address of the DID subject.
    pub account: String,
    /// Hex-encoded W3C DID document (optional, stored on-ledger).
    #[serde(rename = "DIDDocument")]
    pub did_document: Option<String>,
    /// Hex-encoded arbitrary data attached to the DID.
    #[serde(rename = "Data")]
    pub data: Option<String>,
    /// URI pointing to off-ledger DID document or metadata.
    #[serde(rename = "URI")]
    pub uri: Option<String>,

    /// Shared ledger-object metadata (flags, index, previous transaction reference).
    #[serde(flatten)]
    pub common: Common,
}

/// An XRP amount held in escrow, releasable by time or crypto-condition.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Escrow {
    /// r-address of the account that created the escrow.
    pub account: String,
    /// Amount of XRP (in drops) held in escrow.
    pub amount: String,
    /// Ripple epoch time after which the escrow can be cancelled.
    pub cancel_after: Option<u32>,
    /// PREIMAGE-SHA-256 crypto-condition that must be fulfilled to release funds.
    pub condition: Option<String>,
    /// r-address of the intended recipient.
    pub destination: String,
    /// Index into the destination's owner directory.
    pub destination_node: Option<String>,
    /// Destination tag for routing within the destination account.
    pub destination_tag: Option<u32>,
    /// Ripple epoch time after which the escrow can be finished.
    pub finish_after: Option<u32>,
    /// Source tag for routing within the sending account.
    pub source_tag: Option<u32>,

    /// Shared ledger-object metadata (flags, index, previous transaction reference).
    #[serde(flatten)]
    pub common: Common,
}

/// An MPT holding owned by an account for a specific issuance.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MPToken {
    /// r-address of the token holder.
    pub account: String,
    /// 48-character hex ID of the MPT issuance.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mpt_issuance_id: Value,
    /// Current token balance held by this account.
    pub mpt_amount: Value,

    /// Shared ledger-object metadata (flags, index, previous transaction reference).
    #[serde(flatten)]
    pub common: Common,
}

/// An MPT issuance definition created by `issuer`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MPTokenIssuance {
    /// r-address of the account that created the issuance.
    pub issuer: String,
    /// Decimal precision of the token (number of digits after the decimal point).
    pub asset_scale: Option<u8>,
    /// Maximum number of tokens that can ever be minted (string-encoded u64).
    pub maximum_amount: Option<String>,
    /// Total tokens currently in circulation (string-encoded u64).
    pub outstanding_amount: Option<String>,
    /// Transfer fee charged on secondary transfers, in units of 1/100,000.
    pub transfer_fee: Option<u16>,
    /// Hex-encoded arbitrary metadata associated with the issuance.
    #[serde(rename = "MPTokenMetadata")]
    pub mpt_metadata: Option<String>,

    /// Shared ledger-object metadata (flags, index, previous transaction reference).
    #[serde(flatten)]
    pub common: Common,
}

/// An offer to buy or sell a specific NFToken.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NFTokenOffer {
    /// Price offered (XRP drops string or issued-currency object).
    pub amount: Value,
    /// If set, only this r-address may accept the offer.
    pub destination: Option<String>,
    /// Ripple epoch time after which the offer is no longer valid.
    pub expiration: Option<u32>,
    /// 256-bit hex identifier of the NFToken being offered.
    #[serde(rename = "NFTokenID")]
    pub nftoken_id: String,
    /// Index into the NFToken offer directory.
    #[serde(rename = "NFTokenOfferNode")]
    pub nftoken_offer_node: String,
    /// r-address of the account that created this offer.
    pub owner: String,

    /// Shared ledger-object metadata (flags, index, previous transaction reference).
    #[serde(flatten)]
    pub common: Common,
}

/// A page of up to 32 NFTokens stored in an account's NFToken directory.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NFTokenPage {
    /// Low boundary of the next page's NFToken IDs, used for pagination.
    pub next_page_min: Option<String>,
    /// Array of NFToken objects on this page.
    #[serde(rename = "NFTokens")]
    pub nftokens: Value,
    /// High boundary of the previous page's NFToken IDs, used for pagination.
    pub previous_page_min: Option<String>,

    /// Shared ledger-object metadata (flags, index, previous transaction reference).
    #[serde(flatten)]
    pub common: Common,
}

/// A DEX offer to exchange `taker_pays` for `taker_gets`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Offer {
    /// r-address of the account that placed the offer.
    pub account: String,
    /// Hash of the order-book directory this offer belongs to.
    pub book_directory: String,
    /// Index of this offer within its order-book directory page.
    pub book_node: String,
    /// Ripple epoch time after which the offer is automatically removed.
    pub expiration: Option<u32>,
    /// Sequence number of the OfferCreate transaction that created this offer.
    pub sequence: u32,
    /// Amount the taker must pay (what the offer creator wants to receive).
    pub taker_pays: Amount,
    /// Amount the taker receives (what the offer creator is selling).
    pub taker_gets: Amount,

    /// Shared ledger-object metadata (flags, index, previous transaction reference).
    #[serde(flatten)]
    pub common: Common,
}

/// A price oracle entry publishing one or more asset price feeds on-ledger.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Oracle {
    /// r-address of the account that controls this oracle.
    pub account: String,
    /// Unique identifier for this oracle document within the account.
    #[serde(rename = "OracleDocumentID")]
    pub oracle_document_id: u32,
    /// Descriptive asset class (e.g. "currency", "commodity").
    pub asset_class: Option<String>,
    /// Ripple epoch time of the most recent price update.
    pub last_update_time: u32,
    /// Array of price data entries, each containing a base/quote asset pair and price.
    pub price_data_series: Value,
    /// Human-readable name of the data provider.
    pub provider: Option<String>,
    /// URI linking to additional information about this oracle.
    #[serde(rename = "URI")]
    pub uri: Option<String>,

    /// Shared ledger-object metadata (flags, index, previous transaction reference).
    #[serde(flatten)]
    pub common: Common,
}

/// A unidirectional payment channel funded by `account` for streaming payments.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PayChannel {
    /// r-address of the channel funder (source).
    pub account: String,
    /// Total XRP allocated to this channel.
    pub amount: Amount,
    /// XRP already delivered to the destination via claims.
    pub balance: Amount,
    /// Ripple epoch time after which the channel can be force-closed.
    pub cancel_after: Option<u32>,
    /// r-address of the payment recipient.
    pub destination: String,
    /// Destination tag for routing within the destination account.
    pub destination_tag: Option<u32>,
    /// Index into the destination's owner directory.
    pub destination_node: Option<String>,
    /// Ripple epoch time after which the channel expires if not renewed.
    pub expiration: Option<u32>,
    /// Hex-encoded 33-byte public key used to verify off-chain payment claims.
    pub public_key: String,
    /// Minimum time in seconds the source must wait after requesting closure.
    pub settle_delay: u32,
    /// Source tag for routing within the sending account.
    pub source_tag: Option<u32>,

    /// Shared ledger-object metadata (flags, index, previous transaction reference).
    #[serde(flatten)]
    pub common: Common,
}

/// A trust line between two accounts, tracking the issued-currency balance and limits.
///
/// The "low" side is the account whose r-address sorts lexicographically lower.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RippleState {
    /// Current balance of the trust line (positive = low side holds, negative = high side holds).
    pub balance: Amount,
    /// Maximum balance the high-side account is willing to hold.
    pub high_limit: Amount,
    /// Index into the high-side account's owner directory.
    pub high_node: String,
    /// Quality applied to incoming transfers on the high side (rate in millionths).
    pub high_quality_in: Option<u32>,
    /// Quality applied to outgoing transfers on the high side (rate in millionths).
    pub high_quality_out: Option<u32>,
    /// Number of locked balance entries on this trust line.
    pub lock_count: Option<u32>,
    /// Amount of balance currently locked (e.g. by escrow or AMM).
    pub locked_balance: Option<Amount>,
    /// Maximum balance the low-side account is willing to hold.
    pub low_limit: Amount,
    /// Index into the low-side account's owner directory.
    pub low_node: String,
    /// Quality applied to incoming transfers on the low side (rate in millionths).
    pub low_quality_in: Option<u32>,
    /// Quality applied to outgoing transfers on the low side (rate in millionths).
    pub low_quality_out: Option<u32>,

    /// Shared ledger-object metadata (flags, index, previous transaction reference).
    #[serde(flatten)]
    pub common: Common,
}

/// A multi-signature signer list defining the accounts and quorum for an account.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SignerList {
    /// Ordered list of signers and their weights.
    pub signer_entries: Vec<SignerEntry>,
    /// Always `0` — reserved for future use.
    #[serde(rename = "SignerListID")]
    pub signer_list_id: u32,
    /// Minimum total signer weight required to authorize a transaction.
    pub signer_quorum: u32,

    /// Shared ledger-object metadata (flags, index, previous transaction reference).
    #[serde(flatten)]
    pub common: Common,
}

/// One entry in a [`SignerList`], pairing an account with its signing weight.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SignerEntry {
    /// r-address of the signer.
    pub account: String,
    /// Weight this signer contributes toward the quorum.
    pub signer_weight: u16,
    /// Optional 256-bit locator for wallet software.
    pub wallet_locator: Option<String>,
}

impl SignerEntry {
    /// Creates a new `SignerEntry` with the given account and weight.
    /// Use [`with_wallet_locator`] to set the optional wallet locator.
    ///
    /// [`with_wallet_locator`]: SignerEntry::with_wallet_locator
    pub fn new(account: impl AsRef<str>, signer_weight: u16) -> Self {
        Self {
            account: account.as_ref().to_string(),
            signer_weight,
            wallet_locator: None,
        }
    }

    /// Attaches a 256-bit wallet locator.
    pub fn with_wallet_locator(
        mut self,
        wallet_locator: impl AsRef<str>,
    ) -> Self {
        self.wallet_locator = Some(wallet_locator.as_ref().to_string());
        self
    }
}

/// A sequence-number ticket that reserves a future transaction slot.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Ticket {
    /// r-address of the account that created the ticket.
    pub account: String,
    /// The sequence number set aside for the ticket.
    pub ticket_sequence: u32,

    /// Shared ledger-object metadata (flags, index, previous transaction reference).
    #[serde(flatten)]
    pub common: Common,
}

/// A cross-chain bridge door object managed by `account`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Bridge {
    /// r-address of the bridge door account on this chain.
    pub account: String,
    /// Minimum XRP amount (drops) required to create an account via the bridge.
    pub min_account_create_amount: Option<String>,
    /// XRP reward paid to attestation signers per cross-chain transfer.
    pub signature_reward: String,
    /// Running count of cross-chain claim transactions processed.
    #[serde(rename = "XChainAccountClaimCount")]
    pub xchain_account_claim_count: String,
    /// Running count of cross-chain account-create transactions processed.
    #[serde(rename = "XChainAccountCreateCount")]
    pub xchain_account_create_count: String,
    /// Bridge definition identifying both door accounts and assets.
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge,
    /// The next available cross-chain claim ID.
    #[serde(rename = "XChainClaimID")]
    pub xchain_claim_id: String,

    /// Shared ledger-object metadata (flags, index, previous transaction reference).
    #[serde(flatten)]
    pub common: Common,
}

/// A cross-chain claim ID that collects attestations for a pending bridge transfer.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XChainOwnedClaimID {
    /// r-address of the account that created this claim ID.
    pub account: String,
    /// r-address of the source account on the other chain.
    pub other_chain_source: String,
    /// XRP (drops) paid to attestation signers for this transfer.
    pub signature_reward: String,
    /// Bridge this claim ID belongs to.
    pub xchain_bridge: XChainBridge,
    /// Unique numeric identifier for this cross-chain transfer.
    #[serde(rename = "XChainClaimID")]
    pub xchain_claim_id: String,
    /// Collected attestations from bridge signers.
    pub xchain_claim_attestations: Value,

    /// Shared ledger-object metadata (flags, index, previous transaction reference).
    #[serde(flatten)]
    pub common: Common,
}

/// A cross-chain claim ID for a bridge transfer that creates a new account on the destination chain.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XChainOwnedCreateAccountClaimID {
    /// r-address of the account that initiated this account-create transfer.
    pub account: String,
    /// Sequence counter matching the bridge's `XChainAccountCreateCount` when the transfer was initiated.
    #[serde(rename = "XChainAccountCreateCount")]
    pub xchain_account_create_count: String,
    /// Bridge this claim ID belongs to.
    pub xchain_bridge: XChainBridge,
    /// Collected attestations from bridge signers for the account-create transfer.
    pub xchain_create_account_attestations: Value,

    /// Shared ledger-object metadata (flags, index, previous transaction reference).
    #[serde(flatten)]
    pub common: Common,
}
