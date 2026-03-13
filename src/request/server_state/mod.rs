use serde::{Deserialize, Serialize};

use super::{XrplRequest, XrplResponse};

#[derive(Debug, Default, Serialize)]
pub struct ServerStateRequest;

impl XrplRequest for ServerStateRequest {
    type Response = XrplResponse<ServerStateResult>;
    const COMMAND: &str = "server_state";
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServerStateResult {
    pub state: ServerState,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServerState {
    pub build_version: String,
    pub complete_ledgers: String,
    pub io_latency_ms: u64,
    pub jq_trans_overflow: String,
    pub last_close: LastClose,
    pub load_base: u64,
    pub load_factor: u64,
    pub load_factor_fee_escalation: u64,
    pub load_factor_fee_queue: u64,
    pub load_factor_fee_reference: u64,
    pub load_factor_server: u64,
    pub peer_disconnects: String,
    pub peer_disconnects_resources: String,
    pub peers: u64,
    pub pubkey_node: String,
    pub server_state: String,
    pub server_state_duration_us: String,
    pub state_accounting: StateAccounting,
    pub time: String,
    pub uptime: u64,
    pub validated_ledger: ValidatedLedger,
    pub validation_quorum: u64,
    pub reserve_inc_xrp: Option<u32>,
    pub reserve_base_xrp: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LastClose {
    pub converge_time: u64,
    pub proposers: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StateAccounting {
    pub connected: AccountingState,
    pub disconnected: AccountingState,
    pub full: AccountingState,
    pub syncing: AccountingState,
    pub tracking: AccountingState,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountingState {
    pub duration_us: String,
    pub transitions: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ValidatedLedger {
    pub base_fee: u64,
    pub close_time: u64,
    pub hash: String,
    pub reserve_base: u64,
    pub reserve_inc: u64,
    pub seq: u64,
}
