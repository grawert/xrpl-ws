#![allow(clippy::large_enum_variant)]

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::Amount;

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Transaction {
    pub account: String,
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

    #[serde(flatten)]
    pub transaction_type: TransactionType,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "TransactionType", rename_all_fields = "PascalCase")]
pub enum TransactionType {
    NFTokenAcceptOffer {
        nftoken_sell_offer: Option<String>,
        nftoken_buy_offer: Option<String>,
        nftoken_broker_fee: Option<Amount>,
    },
    NFTokenBurn {
        nftoken_id: String,
        owner: Option<String>,
    },
    NFTokenCancelOffer {
        nftoken_offers: Vec<String>,
    },
    NFTokenCreateOffer {
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
    TrustSet {
        limit_amount: Amount,
        quality_in: Option<u32>,
        quality_out: Option<u32>,
    },
    OfferCreate {
        expiration: Option<u32>,
        offer_sequence: Option<u32>,
        taker_gets: Amount,
        taker_pays: Amount,
    },
    Payment {
        amount: Option<Amount>,
        deliver_max: Option<Amount>,
        deliver_min: Option<Amount>,
        destination: String,
        destination_tag: Option<u32>,
        invoice_id: Option<String>,
        paths: Option<Vec<Vec<PathStep>>>,
        send_max: Option<Amount>,
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

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Signer {
    pub account: String,
    pub txn_signature: String,
    pub signing_pub_key: String,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PathStep {
    pub account: Option<String>,
    pub currency: Option<String>,
    pub issuer: Option<String>,
}

pub trait SigningContext {
    type Error;
    fn sign_transaction(&self, tx: &Transaction)
    -> Result<String, Self::Error>;
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
