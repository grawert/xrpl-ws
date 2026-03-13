use serde::{Deserialize, Serialize};

use super::{XrplRequest, XrplResponse};

#[derive(Debug, Default, Serialize)]
pub struct FeeRequest;

impl XrplRequest for FeeRequest {
    type Response = XrplResponse<FeeResult>;
    const COMMAND: &str = "fee";
}

#[derive(Clone, Debug, Deserialize)]
pub struct FeeResult {
    pub drops: FeeDrops,
    pub levels: FeeLevels,
    pub ledger_current_index: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FeeDrops {
    pub base_fee: String,
    pub minimum_fee: String,
    pub median_fee: String,
    pub open_ledger_fee: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FeeLevels {
    pub median_level: String,
    pub minimum_level: String,
    pub open_ledger_level: String,
}
