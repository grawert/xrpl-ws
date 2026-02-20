use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};

#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct AccountNftsRequest {
    pub account: String,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<Value>,
    pub limit: Option<u32>,
    pub marker: Option<Value>,
}

impl XrplRequest for AccountNftsRequest {
    type Response = XrplResponse<AccountNftsResponse>;
    const COMMAND: &'static str = "account_nfts";
}

#[derive(Debug, Deserialize)]
pub struct AccountNftsResponse {
    pub account: String,
    pub account_nfts: Vec<AccountNFToken>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<u32>,
    pub ledger_current_index: Option<u32>,
    pub validated: Option<bool>,
    pub marker: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccountNFToken {
    pub flags: u32,
    pub issuer: String,
    #[serde(rename = "NFTokenID")]
    pub nftoken_id: String,
    #[serde(rename = "NFTokenTaxon")]
    pub nftoken_taxon: u32,
    #[serde(rename = "URI")]
    pub uri: Option<String>,
    #[serde(rename = "nft_serial")]
    pub nft_serial: u32,
}
