#![allow(clippy::large_enum_variant)]

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{Amount, Asset, SignerEntry, AuthAccountWrapper};

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Transaction {
    pub account: String,
    #[serde(rename = "AccountTxnID")]
    pub account_txn_id: Option<String>,
    pub fee: String,
    pub flags: Option<u32>,
    pub last_ledger_sequence: Option<u32>,
    pub memos: Option<Vec<MemoWrapper>>,
    pub sequence: u32,
    pub signers: Option<Vec<SignerWrapper>>,
    pub source_tag: Option<u32>,
    pub ticket_sequence: Option<u32>,
    pub signing_pub_key: Option<String>,
    pub txn_signature: Option<String>,
    pub hash: Option<String>,
    pub date: Option<u32>,

    #[serde(flatten)]
    pub transaction_type: TransactionType,
}

impl Transaction {
    /// Attaches multiple signatures to the transaction and prepares it for multi-signing.
    pub fn add_signatures(&mut self, mut signers: Vec<SignerWrapper>) {
        signers.sort_by(|a, b| a.signer.account.cmp(&b.signer.account));
        self.signers = Some(signers);
        self.signing_pub_key = Some("".to_string());
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "TransactionType", rename_all_fields = "PascalCase")]
pub enum TransactionType {
    AccountDelete {
        destination: String,
        destination_tag: Option<u32>,
        #[serde(rename = "CredentialIDs")]
        credential_ids: Option<Vec<String>>,
    },

    AccountSet {
        clear_flag: Option<i64>,
        domain: Option<String>,
        email_hash: Option<String>,
        message_key: Option<String>,
        set_flag: Option<u32>,
        transfer_rate: Option<u32>,
        tick_size: Option<u32>,
        nftoken_minter: Option<String>,
    },

    #[serde(rename = "AMMBid")]
    AMMBid {
        #[serde(rename = "Asset")]
        asset: Asset,
        #[serde(rename = "Asset2")]
        asset2: Asset,
        #[serde(rename = "BidMin")]
        bid_min: Option<Amount>,
        #[serde(rename = "BidMax")]
        bid_max: Option<Amount>,
        #[serde(rename = "AuthAccounts")]
        auth_accounts: Option<Vec<AuthAccountWrapper>>,
    },
    #[serde(rename = "AMMClawback")]
    AMMClawback {
        #[serde(rename = "Asset")]
        asset: Asset,
        #[serde(rename = "Asset2")]
        asset2: Asset,
        #[serde(rename = "Amount")]
        amount: Option<Amount>,
        #[serde(rename = "Holder")]
        holder: String,
    },
    #[serde(rename = "AMMCreate")]
    AMMCreate {
        #[serde(rename = "Amount")]
        amount: Amount,
        #[serde(rename = "Amount2")]
        amount2: Amount,
        #[serde(rename = "TradingFee")]
        trading_fee: u16,
    },
    #[serde(rename = "AMMDelete")]
    AMMDelete {
        #[serde(rename = "Asset")]
        asset: Asset,
        #[serde(rename = "Asset2")]
        asset2: Asset,
    },
    #[serde(rename = "AMMDeposit")]
    AMMDeposit {
        #[serde(rename = "Asset")]
        asset: Asset,
        #[serde(rename = "Asset2")]
        asset2: Asset,
        #[serde(rename = "Amount")]
        amount: Option<Amount>,
        #[serde(rename = "Amount2")]
        amount2: Option<Amount>,
        #[serde(rename = "EPrice")]
        e_price: Option<Amount>,
        #[serde(rename = "LPTokenOut")]
        lp_token_out: Option<Amount>,
        #[serde(rename = "TradingFee")]
        trading_fee: Option<u16>,
    },
    #[serde(rename = "AMMVote")]
    AMMVote {
        #[serde(rename = "Asset")]
        asset: Asset,
        #[serde(rename = "Asset2")]
        asset2: Asset,
        #[serde(rename = "TradingFee")]
        trading_fee: u16,
    },
    #[serde(rename = "AMMWithdraw")]
    AMMWithdraw {
        #[serde(rename = "Asset")]
        asset: Asset,
        #[serde(rename = "Asset2")]
        asset2: Asset,
        #[serde(rename = "Amount")]
        amount: Option<Amount>,
        #[serde(rename = "Amount2")]
        amount2: Option<Amount>,
        #[serde(rename = "EPrice")]
        e_price: Option<Amount>,
        #[serde(rename = "LPTokenIn")]
        lp_token_in: Option<Amount>,
    },

    CheckCancel {
        #[serde(rename = "CheckID")]
        check_id: String,
    },
    CheckCash {
        #[serde(rename = "CheckID")]
        check_id: String,
        amount: Option<Amount>,
        deliver_min: Option<Amount>,
    },
    CheckCreate {
        destination: String,
        send_max: Amount,
        destination_tag: Option<u32>,
        expiration: Option<u32>,
        #[serde(rename = "InvoiceID")]
        invoice_id: Option<String>,
    },

    Clawback {
        amount: Amount,
        holder: Option<String>,
    },

    CredentialAccept {
        credential_type: Option<String>,
        issuer: Option<String>,
    },
    CredentialCreate {
        credential_type: Option<String>,
        subject: Option<String>,
        expiration: Option<u32>,
    },
    CredentialDelete {
        credential_type: Option<String>,
        subject: Option<String>,
    },

    DepositPreauth {
        authorize: Option<String>,
        unauthorize: Option<String>,
    },

    DIDDelete {
        // This transaction only uses common fields
    },
    DIDSet {
        #[serde(rename = "DIDDocument")]
        did_document: Option<String>,
        #[serde(rename = "Data")]
        data: Option<String>,
        #[serde(rename = "URI")]
        uri: Option<String>,
    },

    EscrowCancel {
        owner: String,
        offer_sequence: u32,
    },
    EscrowCreate {
        amount: Amount,
        destination: String,
        cancel_after: Option<u32>,
        finish_after: Option<u32>,
        condition: Option<String>,
        destination_tag: Option<u32>,
    },
    EscrowFinish {
        owner: String,
        offer_sequence: u32,
        condition: Option<String>,
        fulfillment: Option<String>,
    },

    NFTokenAcceptOffer {
        nftoken_sell_offer: Option<String>,
        nftoken_buy_offer: Option<String>,
        nftoken_broker_fee: Option<Amount>,
    },
    NFTokenBurn {
        #[serde(rename = "NFTokenID")]
        nftoken_id: String,
        owner: Option<String>,
    },
    NFTokenCancelOffer {
        #[serde(rename = "NFTokenOffers")]
        nftoken_offers: Vec<String>,
    },
    NFTokenCreateOffer {
        #[serde(rename = "NFTokenID")]
        nftoken_id: String,
        amount: Amount,
        owner: Option<String>,
        expiration: Option<u32>,
        destination: Option<String>,
    },
    NFTokenMint {
        nftoken_taxon: u32,
        issuer: Option<String>,
        transfer_fee: Option<u32>,
        uri: Option<String>,
    },

    OfferCancel {
        offer_sequence: u32,
    },

    OfferCreate {
        expiration: Option<u32>,
        offer_sequence: Option<u32>,
        taker_gets: Amount,
        taker_pays: Amount,
    },

    OracleDelete {
        #[serde(rename = "OracleDocumentID")]
        oracle_document_id: u32,
    },
    OracleSet {
        #[serde(rename = "AssetClass")]
        asset_class: Option<String>,
        #[serde(rename = "LastUpdateTime")]
        last_update_time: u32,
        #[serde(rename = "OracleDocumentID")]
        oracle_document_id: u32,
        #[serde(rename = "PriceDataSeries")]
        price_data_series: Vec<PriceDataWrapper>,
        #[serde(rename = "Provider")]
        provider: Option<String>,
        #[serde(rename = "URI")]
        uri: Option<String>,
    },

    Payment {
        amount: Option<Amount>,
        deliver_max: Option<Amount>,
        deliver_min: Option<Amount>,
        destination: String,
        destination_tag: Option<u32>,
        #[serde(rename = "InvoiceID")]
        invoice_id: Option<String>,
        paths: Option<Vec<Vec<PathStep>>>,
        send_max: Option<Amount>,
    },

    PaymentChannelClaim {
        channel: String,
        amount: Option<Amount>,
        balance: Option<Amount>,
        #[serde(rename = "CredentialIDs")]
        credential_ids: Option<Vec<String>>,
        #[serde(rename = "PublicKey")]
        public_key: Option<String>,
        signature: Option<String>,
    },

    PaymentChannelCreate {
        amount: Amount,
        destination: String,
        #[serde(rename = "PublicKey")]
        public_key: String,
        #[serde(rename = "SettleDelay")]
        settle_delay: u32,
        destination_tag: Option<u32>,
        cancel_after: Option<u32>,
    },

    PaymentChannelFund {
        channel: String,
        amount: Amount,
        expiration: Option<u32>,
    },

    SetRegularKey {
        regular_key: Option<String>,
    },

    SignerListSet {
        signer_quorum: u32,
        signer_entries: Option<Vec<SignerEntryWrapper>>,
    },

    TicketCreate {
        ticket_count: u32,
    },

    TrustSet {
        limit_amount: Amount,
        quality_in: Option<u32>,
        quality_out: Option<u32>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MemoWrapper {
    pub memo: Memo,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Memo {
    pub memo_data: Option<String>,
    pub memo_format: Option<String>,
    pub memo_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SignerWrapper {
    pub signer: Signer,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PriceDataWrapper {
    pub price_data: PriceData,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PriceData {
    pub asset_price: Option<String>,
    pub base_asset: String,
    pub quote_asset: String,
    pub scale: Option<u8>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Signer {
    pub account: String,
    pub txn_signature: String,
    pub signing_pub_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SignerEntryWrapper {
    #[serde(rename = "SignerEntry")]
    pub signer_entry: SignerEntry,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PathStep {
    pub account: Option<String>,
    pub currency: Option<String>,
    pub issuer: Option<String>,
}

/// Trait for transaction signing.
///
/// # Example
///
/// ```rust
/// use anyhow::anyhow;
/// use ripple_keypairs::{PrivateKey, PublicKey};
/// use xrpl_mithril_codec::serializer::serialize_tx;
/// use xrpl_mithril_codec::signing::HASH_PREFIX_TRANSACTION_SIGN;
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
///         // Attach the public key to the transaction JSON
///         let mut tx_json = serde_json::to_value(tx)
///             .expect("Failed to convert transaction to json");
///         tx_json["SigningPubKey"] = self.public_key.to_string().into();
///
///         // Serialize to XRPL binary format for signing (canonical = true)
///         let tx_hex = serialize_tx(serde_json::to_string(&tx_json)?, true)
///             .ok_or_else(|| {
///                 anyhow!("Failed to serialize transaction for signing")
///             })?;
///
///         // Prepend the XRPL signing prefix and sign
///         let mut signing_bytes = Vec::with_capacity(4 + tx_hex.len() / 2);
///         signing_bytes.extend_from_slice(&HASH_PREFIX_TRANSACTION_SIGN);
///         signing_bytes.extend_from_slice(&hex::decode(tx_hex)?);
///         let signature = self.private_key.sign(&signing_bytes);
///
///         // Attach signature and serialize the final blob (canonical = false)
///         tx_json["TxnSignature"] = signature.to_string().into();
///         let tx_signed = serialize_tx(serde_json::to_string(&tx_json)?, false)
///             .ok_or_else(|| {
///             anyhow!("Failed to serialize signed transaction")
///         })?;
///
///         Ok(tx_signed)
///     }
/// }
/// ```
pub trait SigningContext {
    type Error;
    /// Produces the final signed transaction hex string.
    fn sign_transaction(&self, tx: &Transaction)
    -> Result<String, Self::Error>;
}

/// Trait for multi-signature transaction signing.
///
/// # Example
///
/// ```rust
/// use anyhow::anyhow;
/// use ripple_keypairs::{PrivateKey, PublicKey};
/// use rippled_binary_codec::serialize::serialize_tx;
/// use xrpl::types::{Transaction, MultiSigningContext, SignerWrapper, Signer};
///
/// const SMT_PREFIX: &str = "534D5400";
///
/// struct Wallet {
///     public_key: PublicKey,
///     private_key: PrivateKey,
/// }
///
/// impl Wallet {
///     pub fn finalize_multi_signed(&self, tx: &Transaction) -> Result<String, anyhow::Error> {
///         let mut tx_json = serde_json::to_value(tx).map_err(|e| anyhow!(e))?;
///
///         tx_json["SigningPubKey"] = "".into();
///         if let Some(obj) = tx_json.as_object_mut() {
///             obj.remove("TxnSignature");
///         }
///
///         serialize_tx(serde_json::to_string(&tx_json)?, false)
///             .ok_or_else(|| anyhow!("Multi-sig serialization failed"))
///     }
/// }
///
/// impl MultiSigningContext for Wallet {
///     type Error = anyhow::Error;
///
///     fn sign_as_signer(&self, tx: &Transaction) -> Result<SignerWrapper, Self::Error> {
///         let mut tx_json = serde_json::to_value(tx)
///             .map_err(|e| anyhow!(e.to_string()))?;
///
///         tx_json["SigningPubKey"] = "".into();
///
///         let tx_hex = serialize_tx(serde_json::to_string(&tx_json)?, true)
///             .ok_or_else(|| anyhow!("Serialization failed"))?;
///
///         let address = self.public_key.derive_address();
///         let signing_data = format!("{}{}{}", SMT_PREFIX, tx_hex, hex::encode(address.as_bytes()));
///         let decoded_data = hex::decode(signing_data)?;
///         let signature = self.private_key.sign(&decoded_data);
///
///         Ok(SignerWrapper {
///             signer: Signer {
///                 account: address,
///                 txn_signature: signature.to_string(),
///                 signing_pub_key: self.public_key.to_string(),
///             }
///         })
///     }
/// }
/// ```
pub trait MultiSigningContext {
    type Error;
    fn sign_as_signer(
        &self,
        tx: &Transaction,
    ) -> Result<SignerWrapper, Self::Error>;
}

pub trait Signable {
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
