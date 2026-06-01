use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::types::SignerEntry;

/// Account management transaction types (AccountSet, AccountDelete, etc.).
pub mod account;
/// AMM transaction types (AMMCreate, AMMDeposit, AMMWithdraw, etc.).
pub mod amm;
/// Clawback transaction type.
pub mod clawback;
/// Credential transaction types (CredentialCreate, CredentialAccept, CredentialDelete).
pub mod credential;
/// DID transaction types (DIDSet, DIDDelete).
pub mod did;
/// Escrow transaction types (EscrowCreate, EscrowFinish, EscrowCancel).
pub mod escrow;
/// Multi-Purpose Token transaction types (MPTokenIssuanceCreate, MPTokenAuthorize, etc.).
pub mod mpt;
/// NFToken transaction types (NFTokenMint, NFTokenBurn, NFTokenCreateOffer, etc.).
pub mod nft;
/// DEX offer transaction types (OfferCreate, OfferCancel).
pub mod offer;
/// Price oracle transaction types (OracleSet, OracleDelete).
pub mod oracle;
/// Payment and check transaction types (Payment, CheckCreate, CheckCash, CheckCancel).
pub mod payment;
/// Payment channel transaction types (PaymentChannelCreate, PaymentChannelFund, PaymentChannelClaim).
pub mod payment_channel;
/// TrustSet transaction type.
pub mod trust_set;
/// Cross-chain bridge transaction types (XChainCreateBridge, XChainCommit, XChainClaim, etc.).
pub mod xchain;

pub use account::{
    AccountDelete, AccountSet, DepositPreauth, SetRegularKey, SignerListSet,
    TicketCreate,
};
pub use amm::{
    AMMBid, AMMClawback, AMMCreate, AMMDelete, AMMDeposit, AMMDepositFlags,
    AMMVote, AMMWithdraw, AMMWithdrawFlags,
};
pub use clawback::Clawback;
pub use credential::{CredentialAccept, CredentialCreate, CredentialDelete};
pub use did::{DIDDelete, DIDSet};
pub use escrow::{EscrowCancel, EscrowCreate, EscrowFinish};
pub use mpt::{
    MPTokenAuthorize, MPTokenAuthorizeFlags, MPTokenIssuanceCreate,
    MPTokenIssuanceCreateFlags, MPTokenIssuanceDestroy, MPTokenIssuanceSet,
    MPTokenIssuanceSetAction,
};
pub use nft::{
    NFTokenAcceptOffer, NFTokenBurn, NFTokenCancelOffer, NFTokenCreateOffer,
    NFTokenCreateOfferFlags, NFTokenMint, NFTokenMintFlags,
};
pub use offer::{OfferCancel, OfferCreate, OfferCreateFlags};
pub use oracle::{OracleDelete, OracleSet, PriceData, PriceDataWrapper};
pub use payment::{
    CheckCancel, CheckCash, CheckCreate, PathStep, Payment, PaymentFlag,
    PaymentFlags,
};
pub use payment_channel::{
    PaymentChannelClaim, PaymentChannelClaimAction, PaymentChannelCreate,
    PaymentChannelFund,
};
pub use trust_set::{TrustSet, TrustSetFlags};
pub use xchain::{
    XChainAccountCreateCommit, XChainAddAccountCreateAttestation,
    XChainAddClaimAttestation, XChainClaim, XChainCommit, XChainCreateBridge,
    XChainCreateClaimID, XChainModifyBridge, XChainModifyBridgeFlags,
};

/// Discriminated union over every XRPL transaction type.
///
/// Each variant wraps the type-specific fields for that transaction kind.
/// Use the typed accessor methods on [`Transaction`] (e.g. `as_payment()`) to
/// borrow the inner fields without matching manually.
///
/// The enum is `#[non_exhaustive]` so that new XRPL amendments can be
/// represented by the `Unknown` catch-all without breaking existing match arms.
///
/// # Examples
///
/// ```rust
/// use xrpl::types::TransactionType;
/// // Typically obtained by deserializing a Transaction from a WebSocket message.
/// ```
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum TransactionType {
    /// Remove an account from the ledger permanently.
    AccountDelete(account::AccountDelete),
    /// Modify account settings and flags.
    AccountSet(account::AccountSet),
    /// Bid on the AMM continuous auction slot for a discounted trading fee.
    AMMBid(amm::AMMBid),
    /// Reclaim tokens issued via an AMM from an unauthorized holder.
    AMMClawback(amm::AMMClawback),
    /// Create a new AMM pool for two assets.
    AMMCreate(amm::AMMCreate),
    /// Delete an empty AMM pool.
    AMMDelete(amm::AMMDelete),
    /// Add liquidity to an AMM pool in exchange for LP tokens.
    AMMDeposit(amm::AMMDeposit),
    /// Cast a vote for the AMM pool trading fee.
    AMMVote(amm::AMMVote),
    /// Remove liquidity from an AMM pool by redeeming LP tokens.
    AMMWithdraw(amm::AMMWithdraw),
    /// Cancel an outstanding check without cashing it.
    CheckCancel(payment::CheckCancel),
    /// Cash a check, transferring funds from the creator to the destination.
    CheckCash(payment::CheckCash),
    /// Create a deferred payment check that the destination can cash later.
    CheckCreate(payment::CheckCreate),
    /// Reclaim issued tokens from a holder's trust line.
    Clawback(clawback::Clawback),
    /// Accept a verifiable credential issued to the signer's account.
    CredentialAccept(credential::CredentialAccept),
    /// Issue a verifiable credential to another account.
    CredentialCreate(credential::CredentialCreate),
    /// Delete a verifiable credential from the ledger.
    CredentialDelete(credential::CredentialDelete),
    /// Grant or revoke deposit pre-authorization for an account.
    DepositPreauth(account::DepositPreauth),
    /// Delete a DID document from the ledger.
    DIDDelete(did::DIDDelete),
    /// Create or update a DID document on the ledger.
    DIDSet(did::DIDSet),
    /// Cancel a time-locked or condition-locked escrow.
    EscrowCancel(escrow::EscrowCancel),
    /// Create a time-locked or condition-locked XRP escrow.
    EscrowCreate(escrow::EscrowCreate),
    /// Release funds from an escrow once conditions are met.
    EscrowFinish(escrow::EscrowFinish),
    /// Authorize or un-authorize an MPT holder.
    MPTokenAuthorize(mpt::MPTokenAuthorize),
    /// Create a new MPT issuance definition.
    MPTokenIssuanceCreate(mpt::MPTokenIssuanceCreate),
    /// Destroy an MPT issuance that has no outstanding tokens.
    MPTokenIssuanceDestroy(mpt::MPTokenIssuanceDestroy),
    /// Update flags or properties of an MPT issuance.
    MPTokenIssuanceSet(mpt::MPTokenIssuanceSet),
    /// Accept a buy or sell offer for an NFToken.
    NFTokenAcceptOffer(nft::NFTokenAcceptOffer),
    /// Destroy an NFToken owned by the signer.
    NFTokenBurn(nft::NFTokenBurn),
    /// Cancel one or more NFToken buy or sell offers.
    NFTokenCancelOffer(nft::NFTokenCancelOffer),
    /// Create an offer to buy or sell an NFToken.
    NFTokenCreateOffer(nft::NFTokenCreateOffer),
    /// Mint a new NFToken into the signer's collection.
    NFTokenMint(nft::NFTokenMint),
    /// Cancel an existing DEX offer.
    OfferCancel(offer::OfferCancel),
    /// Place a new DEX offer to exchange one asset for another.
    OfferCreate(offer::OfferCreate),
    /// Delete a price oracle entry from the ledger.
    OracleDelete(oracle::OracleDelete),
    /// Create or update a price oracle entry.
    OracleSet(oracle::OracleSet),
    /// Transfer XRP or issued tokens from one account to another.
    Payment(payment::Payment),
    /// Redeem a signed claim from a payment channel.
    PaymentChannelClaim(payment_channel::PaymentChannelClaim),
    /// Open a new unidirectional payment channel.
    PaymentChannelCreate(payment_channel::PaymentChannelCreate),
    /// Add more XRP to an existing payment channel.
    PaymentChannelFund(payment_channel::PaymentChannelFund),
    /// Assign or remove an alternate signing key for an account.
    SetRegularKey(account::SetRegularKey),
    /// Create, replace, or delete a multi-signature signer list.
    SignerListSet(account::SignerListSet),
    /// Reserve one or more sequence-number tickets for future use.
    TicketCreate(account::TicketCreate),
    /// Create or modify a trust line for an issued currency.
    TrustSet(trust_set::TrustSet),
    /// Lock XRP on the locking chain to initiate a cross-chain account creation.
    XChainAccountCreateCommit(xchain::XChainAccountCreateCommit),
    /// Submit a signer attestation for a cross-chain account-create transfer.
    XChainAddAccountCreateAttestation(
        xchain::XChainAddAccountCreateAttestation,
    ),
    /// Submit a signer attestation for a cross-chain asset transfer.
    XChainAddClaimAttestation(xchain::XChainAddClaimAttestation),
    /// Claim funds on the destination chain of a cross-chain transfer.
    XChainClaim(xchain::XChainClaim),
    /// Lock assets on the source chain to initiate a cross-chain transfer.
    XChainCommit(xchain::XChainCommit),
    /// Register a new cross-chain bridge on the ledger.
    XChainCreateBridge(xchain::XChainCreateBridge),
    /// Reserve a cross-chain claim ID for an incoming transfer.
    XChainCreateClaimID(xchain::XChainCreateClaimID),
    /// Update parameters of an existing cross-chain bridge.
    XChainModifyBridge(xchain::XChainModifyBridge),
    /// Catch-all for transaction types not yet modelled (e.g., new amendments).
    Unknown {
        /// The raw `TransactionType` string from the wire format.
        name: String,
        /// All fields from the original JSON object, preserved for inspection.
        extra: serde_json::Map<String, serde_json::Value>,
    },
}

/// Common fields shared by every XRPL transaction, plus the type-specific payload.
///
/// Build a `Transaction` through the typed builder API (e.g. `PaymentBuilder`)
/// and sign it with a [`SigningContext`] via [`Signable::sign_with`].
/// For multi-signature workflows, collect [`SignerWrapper`]s and attach them
/// with [`Transaction::add_signatures`].
///
/// # Examples
///
/// ```rust
/// use xrpl::types::Transaction;
/// // Typically obtained by deserializing a WebSocket transaction message.
/// ```
#[derive(Debug, Clone)]
pub struct Transaction {
    /// r-address of the account initiating the transaction.
    pub account: String,
    /// Hash of a previous transaction from this account used for mutual exclusion.
    pub account_txn_id: Option<String>,
    /// Transaction cost in XRP drops (string-encoded).
    pub fee: String,
    /// Bitfield of transaction flags specific to the transaction type.
    pub flags: Option<u32>,
    /// The transaction is invalid and must not be applied after this ledger sequence.
    pub last_ledger_sequence: Option<u32>,
    /// Optional arbitrary data attached to the transaction.
    pub memos: Option<Vec<MemoWrapper>>,
    /// Account sequence number; must match the account's current sequence.
    pub sequence: u32,
    /// Multi-signature entries; present instead of `txn_signature` for multi-sig transactions.
    pub signers: Option<Vec<SignerWrapper>>,
    /// u32 tag identifying the originating party within the sending account.
    pub source_tag: Option<u32>,
    /// Ticket sequence number used in place of `sequence` when tickets are enabled.
    pub ticket_sequence: Option<u32>,
    /// Hex-encoded public key used for single signing; empty string for multi-sig.
    pub signing_pub_key: Option<String>,
    /// DER-encoded hex signature over the canonical serialization of this transaction.
    pub txn_signature: Option<String>,
    /// Transaction hash assigned by the ledger after validation.
    pub hash: Option<String>,
    /// Ledger close time in Ripple epoch seconds (seconds since 2000-01-01T00:00:00 UTC).
    pub date: Option<u32>,
    /// Type-specific payload for this transaction.
    pub transaction_type: TransactionType,
}

macro_rules! impl_tx_accessors {
    ( $( ($accessor:ident, $variant:ident, $ty:ty) ),+ $(,)? ) => {
        impl Transaction {
            /// Returns the `TransactionType` field value as a string slice,
            /// matching the name used in the XRPL wire format.
            pub fn transaction_type_name(&self) -> &str {
                match &self.transaction_type {
                    $( TransactionType::$variant(_) => stringify!($variant), )+
                    TransactionType::Unknown { name, .. } => name.as_str(),
                }
            }

            $(
                /// Returns the type-specific fields if this transaction matches the
                /// corresponding [`TransactionType`] variant, or `None` otherwise.
                pub fn $accessor(&self) -> Option<&$ty> {
                    if let TransactionType::$variant(fields) = &self.transaction_type {
                        Some(fields)
                    } else {
                        None
                    }
                }
            )+
        }
    };
}

impl_tx_accessors! {
    (as_account_delete,                     AccountDelete,                    account::AccountDelete),
    (as_account_set,                        AccountSet,                       account::AccountSet),
    (as_amm_bid,                            AMMBid,                           amm::AMMBid),
    (as_amm_clawback,                       AMMClawback,                      amm::AMMClawback),
    (as_amm_create,                         AMMCreate,                        amm::AMMCreate),
    (as_amm_delete,                         AMMDelete,                        amm::AMMDelete),
    (as_amm_deposit,                        AMMDeposit,                       amm::AMMDeposit),
    (as_amm_vote,                           AMMVote,                          amm::AMMVote),
    (as_amm_withdraw,                       AMMWithdraw,                      amm::AMMWithdraw),
    (as_check_cancel,                       CheckCancel,                      payment::CheckCancel),
    (as_check_cash,                         CheckCash,                        payment::CheckCash),
    (as_check_create,                       CheckCreate,                      payment::CheckCreate),
    (as_clawback,                           Clawback,                         clawback::Clawback),
    (as_credential_accept,                  CredentialAccept,                 credential::CredentialAccept),
    (as_credential_create,                  CredentialCreate,                 credential::CredentialCreate),
    (as_credential_delete,                  CredentialDelete,                 credential::CredentialDelete),
    (as_deposit_preauth,                    DepositPreauth,                   account::DepositPreauth),
    (as_did_delete,                         DIDDelete,                        did::DIDDelete),
    (as_did_set,                            DIDSet,                           did::DIDSet),
    (as_escrow_cancel,                      EscrowCancel,                     escrow::EscrowCancel),
    (as_escrow_create,                      EscrowCreate,                     escrow::EscrowCreate),
    (as_escrow_finish,                      EscrowFinish,                     escrow::EscrowFinish),
    (as_mpt_authorize,                      MPTokenAuthorize,                 mpt::MPTokenAuthorize),
    (as_mpt_issuance_create,                MPTokenIssuanceCreate,            mpt::MPTokenIssuanceCreate),
    (as_mpt_issuance_destroy,               MPTokenIssuanceDestroy,           mpt::MPTokenIssuanceDestroy),
    (as_mpt_issuance_set,                   MPTokenIssuanceSet,               mpt::MPTokenIssuanceSet),
    (as_nftoken_accept_offer,               NFTokenAcceptOffer,               nft::NFTokenAcceptOffer),
    (as_nftoken_burn,                       NFTokenBurn,                      nft::NFTokenBurn),
    (as_nftoken_cancel_offer,               NFTokenCancelOffer,               nft::NFTokenCancelOffer),
    (as_nftoken_create_offer,               NFTokenCreateOffer,               nft::NFTokenCreateOffer),
    (as_nftoken_mint,                       NFTokenMint,                      nft::NFTokenMint),
    (as_offer_cancel,                       OfferCancel,                      offer::OfferCancel),
    (as_offer_create,                       OfferCreate,                      offer::OfferCreate),
    (as_oracle_delete,                      OracleDelete,                     oracle::OracleDelete),
    (as_oracle_set,                         OracleSet,                        oracle::OracleSet),
    (as_payment,                            Payment,                          payment::Payment),
    (as_payment_channel_claim,              PaymentChannelClaim,              payment_channel::PaymentChannelClaim),
    (as_payment_channel_create,             PaymentChannelCreate,             payment_channel::PaymentChannelCreate),
    (as_payment_channel_fund,               PaymentChannelFund,               payment_channel::PaymentChannelFund),
    (as_set_regular_key,                    SetRegularKey,                    account::SetRegularKey),
    (as_signer_list_set,                    SignerListSet,                    account::SignerListSet),
    (as_ticket_create,                      TicketCreate,                     account::TicketCreate),
    (as_trust_set,                          TrustSet,                         trust_set::TrustSet),
    (as_xchain_account_create_commit,       XChainAccountCreateCommit,        xchain::XChainAccountCreateCommit),
    (as_xchain_add_account_create_attestation,   XChainAddAccountCreateAttestation, xchain::XChainAddAccountCreateAttestation),
    (as_xchain_add_claim_attestation,       XChainAddClaimAttestation,        xchain::XChainAddClaimAttestation),
    (as_xchain_claim,                       XChainClaim,                      xchain::XChainClaim),
    (as_xchain_commit,                      XChainCommit,                     xchain::XChainCommit),
    (as_xchain_create_bridge,               XChainCreateBridge,               xchain::XChainCreateBridge),
    (as_xchain_create_claim_id,             XChainCreateClaimID,              xchain::XChainCreateClaimID),
    (as_xchain_modify_bridge,               XChainModifyBridge,               xchain::XChainModifyBridge),
}

impl Serialize for Transaction {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        use serde::ser::Error;
        use serde_json::Value;

        // Serialize common fields via a temporary helper struct that derives Serialize.
        // Alternatively, build the map field by field.
        let mut map = serde_json::Map::new();

        macro_rules! insert {
            ($key:expr, $val:expr) => {
                map.insert(
                    $key.to_string(),
                    serde_json::to_value($val).map_err(Error::custom)?,
                );
            };
        }
        macro_rules! insert_if_some {
            ($key:expr, $val:expr) => {
                if let Some(ref v) = $val {
                    map.insert(
                        $key.to_string(),
                        serde_json::to_value(v).map_err(Error::custom)?,
                    );
                }
            };
        }

        insert!("Account", &self.account);
        insert!("Fee", &self.fee);
        insert!("Sequence", self.sequence);
        insert_if_some!("AccountTxnID", self.account_txn_id);
        insert_if_some!("Flags", self.flags);
        insert_if_some!("LastLedgerSequence", self.last_ledger_sequence);
        insert_if_some!("Memos", self.memos);
        insert_if_some!("Signers", self.signers);
        insert_if_some!("SourceTag", self.source_tag);
        insert_if_some!("TicketSequence", self.ticket_sequence);
        insert_if_some!("SigningPubKey", self.signing_pub_key);
        insert_if_some!("TxnSignature", self.txn_signature);
        insert_if_some!("Hash", self.hash);
        insert_if_some!("Date", self.date);

        // Discriminator
        map.insert(
            "TransactionType".to_string(),
            Value::String(self.transaction_type_name().to_string()),
        );

        // Type-specific fields
        macro_rules! merge {
            ($fields:expr) => {{
                let v = serde_json::to_value($fields).map_err(Error::custom)?;
                if let Some(obj) = v.as_object() {
                    for (k, v) in obj {
                        map.insert(k.clone(), v.clone());
                    }
                }
            }};
        }

        match &self.transaction_type {
            TransactionType::AccountDelete(f) => merge!(f),
            TransactionType::AccountSet(f) => merge!(f),
            TransactionType::AMMBid(f) => merge!(f),
            TransactionType::AMMClawback(f) => merge!(f),
            TransactionType::AMMCreate(f) => merge!(f),
            TransactionType::AMMDelete(f) => merge!(f),
            TransactionType::AMMDeposit(f) => merge!(f),
            TransactionType::AMMVote(f) => merge!(f),
            TransactionType::AMMWithdraw(f) => merge!(f),
            TransactionType::CheckCancel(f) => merge!(f),
            TransactionType::CheckCash(f) => merge!(f),
            TransactionType::CheckCreate(f) => merge!(f),
            TransactionType::Clawback(f) => merge!(f),
            TransactionType::CredentialAccept(f) => merge!(f),
            TransactionType::CredentialCreate(f) => merge!(f),
            TransactionType::CredentialDelete(f) => merge!(f),
            TransactionType::DepositPreauth(f) => merge!(f),
            TransactionType::DIDDelete(f) => merge!(f),
            TransactionType::DIDSet(f) => merge!(f),
            TransactionType::EscrowCancel(f) => merge!(f),
            TransactionType::EscrowCreate(f) => merge!(f),
            TransactionType::EscrowFinish(f) => merge!(f),
            TransactionType::MPTokenAuthorize(f) => merge!(f),
            TransactionType::MPTokenIssuanceCreate(f) => merge!(f),
            TransactionType::MPTokenIssuanceDestroy(f) => merge!(f),
            TransactionType::MPTokenIssuanceSet(f) => merge!(f),
            TransactionType::NFTokenAcceptOffer(f) => merge!(f),
            TransactionType::NFTokenBurn(f) => merge!(f),
            TransactionType::NFTokenCancelOffer(f) => merge!(f),
            TransactionType::NFTokenCreateOffer(f) => merge!(f),
            TransactionType::NFTokenMint(f) => merge!(f),
            TransactionType::OfferCancel(f) => merge!(f),
            TransactionType::OfferCreate(f) => merge!(f),
            TransactionType::OracleDelete(f) => merge!(f),
            TransactionType::OracleSet(f) => merge!(f),
            TransactionType::Payment(f) => merge!(f),
            TransactionType::PaymentChannelClaim(f) => merge!(f),
            TransactionType::PaymentChannelCreate(f) => merge!(f),
            TransactionType::PaymentChannelFund(f) => merge!(f),
            TransactionType::SetRegularKey(f) => merge!(f),
            TransactionType::SignerListSet(f) => merge!(f),
            TransactionType::TicketCreate(f) => merge!(f),
            TransactionType::TrustSet(f) => merge!(f),
            TransactionType::XChainAccountCreateCommit(f) => merge!(f),
            TransactionType::XChainAddAccountCreateAttestation(f) => merge!(f),
            TransactionType::XChainAddClaimAttestation(f) => merge!(f),
            TransactionType::XChainClaim(f) => merge!(f),
            TransactionType::XChainCommit(f) => merge!(f),
            TransactionType::XChainCreateBridge(f) => merge!(f),
            TransactionType::XChainCreateClaimID(f) => merge!(f),
            TransactionType::XChainModifyBridge(f) => merge!(f),
            TransactionType::Unknown { extra, .. } => {
                for (k, v) in extra {
                    map.insert(k.clone(), v.clone());
                }
            }
        }

        map.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Transaction {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        use serde::de::Error;

        let value = serde_json::Value::deserialize(deserializer)?;
        let map = value.as_object().ok_or_else(|| {
            Error::custom("Transaction must be a JSON object")
        })?;

        let tx_type =
            map.get("TransactionType").and_then(|v| v.as_str()).ok_or_else(
                || Error::custom("missing or non-string TransactionType"),
            )?;

        macro_rules! extract {
            ($key:expr, $ty:ty) => {
                serde_json::from_value::<$ty>(
                    map.get($key).cloned().unwrap_or(serde_json::Value::Null),
                )
                .map_err(Error::custom)?
            };
        }
        macro_rules! extract_opt {
            ($key:expr, $ty:ty) => {
                map.get($key)
                    .filter(|v| !v.is_null())
                    .map(|v| serde_json::from_value::<$ty>(v.clone()))
                    .transpose()
                    .map_err(Error::custom)?
            };
        }

        // Common fields
        let account = extract!("Account", String);
        let fee = extract!("Fee", String);
        let sequence = extract!("Sequence", u32);
        let account_txn_id = extract_opt!("AccountTxnID", String);
        let flags = extract_opt!("Flags", u32);
        let last_ledger_sequence = extract_opt!("LastLedgerSequence", u32);
        let memos = extract_opt!("Memos", Vec<MemoWrapper>);
        let signers = extract_opt!("Signers", Vec<SignerWrapper>);
        let source_tag = extract_opt!("SourceTag", u32);
        let ticket_sequence = extract_opt!("TicketSequence", u32);
        let signing_pub_key = extract_opt!("SigningPubKey", String);
        let txn_signature = extract_opt!("TxnSignature", String);
        let hash = extract_opt!("Hash", String);
        let date = extract_opt!("Date", u32);

        // Type-specific fields: deserialize from the same full value
        macro_rules! deser_variant {
            ($variant:ident, $ty:ty) => {{
                let fields = serde_json::from_value::<$ty>(value.clone())
                    .map_err(Error::custom)?;
                TransactionType::$variant(fields)
            }};
        }

        let transaction_type = match tx_type {
            "AccountDelete" => {
                deser_variant!(AccountDelete, account::AccountDelete)
            }
            "AccountSet" => deser_variant!(AccountSet, account::AccountSet),
            "AMMBid" => deser_variant!(AMMBid, amm::AMMBid),
            "AMMClawback" => deser_variant!(AMMClawback, amm::AMMClawback),
            "AMMCreate" => deser_variant!(AMMCreate, amm::AMMCreate),
            "AMMDelete" => deser_variant!(AMMDelete, amm::AMMDelete),
            "AMMDeposit" => deser_variant!(AMMDeposit, amm::AMMDeposit),
            "AMMVote" => deser_variant!(AMMVote, amm::AMMVote),
            "AMMWithdraw" => deser_variant!(AMMWithdraw, amm::AMMWithdraw),
            "CheckCancel" => deser_variant!(CheckCancel, payment::CheckCancel),
            "CheckCash" => deser_variant!(CheckCash, payment::CheckCash),
            "CheckCreate" => deser_variant!(CheckCreate, payment::CheckCreate),
            "Clawback" => deser_variant!(Clawback, clawback::Clawback),
            "CredentialAccept" => {
                deser_variant!(CredentialAccept, credential::CredentialAccept)
            }
            "CredentialCreate" => {
                deser_variant!(CredentialCreate, credential::CredentialCreate)
            }
            "CredentialDelete" => {
                deser_variant!(CredentialDelete, credential::CredentialDelete)
            }
            "DepositPreauth" => {
                deser_variant!(DepositPreauth, account::DepositPreauth)
            }
            "DIDDelete" => deser_variant!(DIDDelete, did::DIDDelete),
            "DIDSet" => deser_variant!(DIDSet, did::DIDSet),
            "EscrowCancel" => {
                deser_variant!(EscrowCancel, escrow::EscrowCancel)
            }
            "EscrowCreate" => {
                deser_variant!(EscrowCreate, escrow::EscrowCreate)
            }
            "EscrowFinish" => {
                deser_variant!(EscrowFinish, escrow::EscrowFinish)
            }
            "MPTokenAuthorize" => {
                deser_variant!(MPTokenAuthorize, mpt::MPTokenAuthorize)
            }
            "MPTokenIssuanceCreate" => deser_variant!(
                MPTokenIssuanceCreate,
                mpt::MPTokenIssuanceCreate
            ),
            "MPTokenIssuanceDestroy" => deser_variant!(
                MPTokenIssuanceDestroy,
                mpt::MPTokenIssuanceDestroy
            ),
            "MPTokenIssuanceSet" => {
                deser_variant!(MPTokenIssuanceSet, mpt::MPTokenIssuanceSet)
            }
            "NFTokenAcceptOffer" => {
                deser_variant!(NFTokenAcceptOffer, nft::NFTokenAcceptOffer)
            }
            "NFTokenBurn" => deser_variant!(NFTokenBurn, nft::NFTokenBurn),
            "NFTokenCancelOffer" => {
                deser_variant!(NFTokenCancelOffer, nft::NFTokenCancelOffer)
            }
            "NFTokenCreateOffer" => {
                deser_variant!(NFTokenCreateOffer, nft::NFTokenCreateOffer)
            }
            "NFTokenMint" => deser_variant!(NFTokenMint, nft::NFTokenMint),
            "OfferCancel" => deser_variant!(OfferCancel, offer::OfferCancel),
            "OfferCreate" => deser_variant!(OfferCreate, offer::OfferCreate),
            "OracleDelete" => {
                deser_variant!(OracleDelete, oracle::OracleDelete)
            }
            "OracleSet" => deser_variant!(OracleSet, oracle::OracleSet),
            "Payment" => deser_variant!(Payment, payment::Payment),
            "PaymentChannelClaim" => deser_variant!(
                PaymentChannelClaim,
                payment_channel::PaymentChannelClaim
            ),
            "PaymentChannelCreate" => deser_variant!(
                PaymentChannelCreate,
                payment_channel::PaymentChannelCreate
            ),
            "PaymentChannelFund" => deser_variant!(
                PaymentChannelFund,
                payment_channel::PaymentChannelFund
            ),
            "SetRegularKey" => {
                deser_variant!(SetRegularKey, account::SetRegularKey)
            }
            "SignerListSet" => {
                deser_variant!(SignerListSet, account::SignerListSet)
            }
            "TicketCreate" => {
                deser_variant!(TicketCreate, account::TicketCreate)
            }
            "TrustSet" => deser_variant!(TrustSet, trust_set::TrustSet),
            "XChainAccountCreateCommit" => deser_variant!(
                XChainAccountCreateCommit,
                xchain::XChainAccountCreateCommit
            ),
            "XChainAddAccountCreateAttestation" => deser_variant!(
                XChainAddAccountCreateAttestation,
                xchain::XChainAddAccountCreateAttestation
            ),
            "XChainAddClaimAttestation" => deser_variant!(
                XChainAddClaimAttestation,
                xchain::XChainAddClaimAttestation
            ),
            "XChainClaim" => deser_variant!(XChainClaim, xchain::XChainClaim),
            "XChainCommit" => {
                deser_variant!(XChainCommit, xchain::XChainCommit)
            }
            "XChainCreateBridge" => {
                deser_variant!(XChainCreateBridge, xchain::XChainCreateBridge)
            }
            "XChainCreateClaimID" => {
                deser_variant!(XChainCreateClaimID, xchain::XChainCreateClaimID)
            }
            "XChainModifyBridge" => {
                deser_variant!(XChainModifyBridge, xchain::XChainModifyBridge)
            }
            other => TransactionType::Unknown {
                name: other.to_string(),
                extra: map.clone(),
            },
        };

        Ok(Transaction {
            account,
            account_txn_id,
            fee,
            flags,
            last_ledger_sequence,
            memos,
            sequence,
            signers,
            source_tag,
            ticket_sequence,
            signing_pub_key,
            txn_signature,
            hash,
            date,
            transaction_type,
        })
    }
}

/// Wire-format wrapper that nests a [`Memo`] under the `Memo` key in the `Memos` array.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MemoWrapper {
    /// The contained memo.
    pub memo: Memo,
}

/// Arbitrary data attached to a transaction, hex-encoded on the wire.
///
/// All three fields are hex strings. `MemoType` and `MemoFormat` conventionally
/// hold MIME types or similar descriptors (also hex-encoded).
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Memo {
    /// Hex-encoded memo payload.
    pub memo_data: Option<String>,
    /// Hex-encoded MIME type or format descriptor for `MemoData`.
    pub memo_format: Option<String>,
    /// Hex-encoded identifier for the memo's purpose or category.
    pub memo_type: Option<String>,
}

impl Memo {
    /// Creates a memo carrying only `memo_data`. Chain [`with_format`] and
    /// [`with_type`] to add the optional fields.
    ///
    /// [`with_format`]: Memo::with_format
    /// [`with_type`]: Memo::with_type
    pub fn new(memo_data: impl Into<String>) -> Self {
        Self {
            memo_data: Some(memo_data.into()),
            memo_format: None,
            memo_type: None,
        }
    }

    /// Sets the hex-encoded format descriptor.
    pub fn with_format(mut self, memo_format: impl Into<String>) -> Self {
        self.memo_format = Some(memo_format.into());
        self
    }

    /// Sets the hex-encoded type/category identifier.
    pub fn with_type(mut self, memo_type: impl Into<String>) -> Self {
        self.memo_type = Some(memo_type.into());
        self
    }
}

/// Wire-format wrapper that nests a [`Signer`] under the `Signer` key in the `Signers` array.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SignerWrapper {
    /// The contained signer entry.
    pub signer: Signer,
}

impl From<Signer> for SignerWrapper {
    fn from(signer: Signer) -> Self {
        Self { signer }
    }
}

impl From<SignerWrapper> for Signer {
    fn from(wrapper: SignerWrapper) -> Self {
        wrapper.signer
    }
}

/// One signature in a multi-signed transaction.
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Signer {
    /// r-address of the signing account.
    pub account: String,
    /// DER-encoded hex signature produced by this signer.
    pub txn_signature: String,
    /// Hex-encoded public key used by this signer.
    pub signing_pub_key: String,
}

impl Signer {
    /// Creates a new `Signer` from the account, signature, and public key.
    pub fn new(
        account: impl Into<String>,
        txn_signature: impl Into<String>,
        signing_pub_key: impl Into<String>,
    ) -> Self {
        Self {
            account: account.into(),
            txn_signature: txn_signature.into(),
            signing_pub_key: signing_pub_key.into(),
        }
    }
}

/// Wire-format wrapper that nests a [`SignerEntry`] under the `SignerEntry` key.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SignerEntryWrapper {
    /// The contained signer entry.
    #[serde(rename = "SignerEntry")]
    pub signer_entry: SignerEntry,
}

impl From<SignerEntry> for SignerEntryWrapper {
    fn from(signer_entry: SignerEntry) -> Self {
        Self { signer_entry }
    }
}

/// Trait for transaction signing.
///
/// # Example
///
/// ```rust,no_run
/// use ripple_keypairs::{PrivateKey, PublicKey};
/// use xrpl_mithril::codec::serializer::serialize_json_object;
/// use xrpl_mithril::codec::signing::HASH_PREFIX_TRANSACTION_SIGN;
/// use xrpl::types::{Transaction, SigningContext};
///
/// struct Wallet {
///     public_key: PublicKey,
///     private_key: PrivateKey,
/// }
///
/// impl SigningContext for Wallet {
///     type Error = anyhow::Error;
///
///     fn sign_transaction(
///         &self,
///         tx: &Transaction,
///     ) -> Result<String, Self::Error> {
///         let mut tx_json = serde_json::to_value(tx)
///             .expect("Failed to convert transaction to json");
///         tx_json["SigningPubKey"] = self.public_key.to_string().into();
///
///         let signing_buf = {
///             let map = tx_json.as_object().expect("Transaction should be JSON object");
///             let mut buf = Vec::new();
///             serialize_json_object(map, &mut buf, true)?;
///             buf
///         };
///
///         let mut signing_bytes = Vec::with_capacity(4 + signing_buf.len());
///         signing_bytes.extend_from_slice(&HASH_PREFIX_TRANSACTION_SIGN);
///         signing_bytes.extend_from_slice(&signing_buf);
///         let signature = self.private_key.sign(&signing_bytes);
///         tx_json["TxnSignature"] = signature.to_string().into();
///
///         let map = tx_json.as_object().expect("Transaction should be JSON object");
///         let mut final_buf = Vec::new();
///         serialize_json_object(map, &mut final_buf, false)?;
///
///         Ok(hex::encode(final_buf).to_uppercase())
///     }
/// }
/// ```
pub trait SigningContext {
    /// Error type returned when signing fails.
    type Error;
    /// Produces the final signed transaction hex string.
    fn sign_transaction(&self, tx: &Transaction)
    -> Result<String, Self::Error>;
}

/// Trait for multi-signature transaction signing.
///
/// # Example
///
/// ```rust,no_run
/// use ripple_keypairs::{PrivateKey, PublicKey};
/// use xrpl_mithril::codec::signing::multi_signing_data;
/// use xrpl::types::{Transaction, MultiSigningContext, SignerWrapper, Signer};
///
/// struct Wallet {
///     pub public_key: PublicKey,
///     pub private_key: PrivateKey,
///     /// 20-byte XRPL account ID (RIPEMD160(SHA256(pubkey_bytes)))
///     pub account_id: [u8; 20],
/// }
///
/// impl MultiSigningContext for Wallet {
///     type Error = anyhow::Error;
///
///     fn sign_as_signer(&self, tx: &Transaction) -> Result<SignerWrapper, Self::Error> {
///         let mut tx_json = serde_json::to_value(tx)
///             .expect("Failed to convert transaction to json");
///         tx_json["SigningPubKey"] = "".into();
///
///         let map = tx_json.as_object().expect("Transaction should be JSON object");
///         let signing_bytes = multi_signing_data(map, &self.account_id)?;
///         let signature = self.private_key.sign(&signing_bytes);
///
///         Ok(SignerWrapper {
///             signer: Signer {
///                 account: self.public_key.derive_address(),
///                 txn_signature: signature.to_string(),
///                 signing_pub_key: self.public_key.to_string(),
///             }
///         })
///     }
/// }
/// ```
pub trait MultiSigningContext {
    /// Error type returned when signing fails.
    type Error;
    /// Produce a single [`SignerWrapper`] for `tx`, to be collected with other signers.
    fn sign_as_signer(
        &self,
        tx: &Transaction,
    ) -> Result<SignerWrapper, Self::Error>;
}

/// Enables single-key signing on a [`Transaction`] via `.sign_with(context)`.
pub trait Signable {
    /// Sign the transaction using `context` and return the serialized hex blob.
    fn sign_with<C: SigningContext>(
        &self,
        context: &C,
    ) -> Result<String, C::Error>;
}

impl Signable for Transaction {
    fn sign_with<C: SigningContext>(
        &self,
        context: &C,
    ) -> Result<String, C::Error> {
        context.sign_transaction(self)
    }
}

/// Enables multi-signature signing on a [`Transaction`] via `.sign_as(context)`.
pub trait MultiSignable {
    /// Produce a [`SignerWrapper`] from `context` for this transaction.
    ///
    /// Collect the results from each signer, then pass them to
    /// [`Transaction::add_signatures`] before submitting.
    fn sign_as<C: MultiSigningContext>(
        &self,
        context: &C,
    ) -> Result<SignerWrapper, C::Error>;
}

impl MultiSignable for Transaction {
    fn sign_as<C: MultiSigningContext>(
        &self,
        context: &C,
    ) -> Result<SignerWrapper, C::Error> {
        context.sign_as_signer(self)
    }
}

impl Transaction {
    /// For `TicketCreate` transactions, returns the sequence numbers of all allocated tickets.
    /// Returns `None` for any other transaction type.
    ///
    /// Ticket sequences are assigned deterministically by the protocol: a `TicketCreate`
    /// at `Sequence = S` with `TicketCount = N` allocates `S+1 … S+N`. No extra ledger
    /// query is needed — call this immediately after the transaction is built.
    ///
    /// # Example
    /// ```
    /// # use xrpl::{Client, types::builders::{TicketCreateBuilder, SubmitRequestBuilder}};
    /// # async fn example(client: &Client) -> anyhow::Result<()> {
    /// let tx = TicketCreateBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", 3)
    ///     .fill(client)
    ///     .await?
    ///     .build()?;
    /// let seqs = tx.ticket_sequences().unwrap(); // [S+1, S+2, S+3]
    /// # Ok(())
    /// # }
    /// ```
    pub fn ticket_sequences(&self) -> Option<Vec<u32>> {
        self.as_ticket_create().map(|tc| {
            (1..=tc.ticket_count).map(|i| self.sequence + i).collect()
        })
    }

    /// Appends a single signature and keeps the signer list sorted by account address.
    ///
    /// Accepts a [`Signer`] (or any type implementing `Into<Signer>`) and wraps
    /// it before insertion.
    ///
    /// Sets `SigningPubKey` to `""` on the first call. Collect all signatures
    /// before submitting via [`SubmitMultisignedRequestBuilder`].
    ///
    /// [`SubmitMultisignedRequestBuilder`]: crate::types::builders::SubmitMultisignedRequestBuilder
    pub fn add_signature(&mut self, signer: impl Into<Signer>) {
        let wrapper = SignerWrapper { signer: signer.into() };
        let signers = self.signers.get_or_insert_with(Vec::new);
        let pos = signers
            .partition_point(|s| s.signer.account < wrapper.signer.account);
        signers.insert(pos, wrapper);
        self.signing_pub_key = Some("".to_string());
    }

    /// Attaches all signatures at once and keeps the signer list sorted by account address.
    ///
    /// Accepts any iterable of items convertible into [`Signer`].
    pub fn add_signatures<I, S>(&mut self, signers: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<Signer>,
    {
        let mut wrapped: Vec<SignerWrapper> = signers
            .into_iter()
            .map(|s| SignerWrapper { signer: s.into() })
            .collect();
        wrapped.sort_by(|a, b| a.signer.account.cmp(&b.signer.account));
        self.signers = Some(wrapped);
        self.signing_pub_key = Some("".to_string());
    }
}
