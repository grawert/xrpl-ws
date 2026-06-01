use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};

/// Retrieves a machine-readable summary of the server's state (values in drops, not XRP).
///
/// Prefer `server_state` over `server_info` when parsing programmatically, as numeric
/// fields use integer drops rather than floating-point XRP.
///
/// # Example
/// ```rust
/// use xrpl::request::server_state::ServerStateRequest;
///
/// let request = ServerStateRequest {
///     ledger_index: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct ServerStateRequest {
    /// Provide "current" to query a Clio server.
    pub ledger_index: Option<Value>,
}

impl XrplRequest for ServerStateRequest {
    type Response = XrplResponse<ServerStateResult>;
    const COMMAND: &str = "server_state";
}

/// Response to a `server_state` request.
#[derive(Clone, Debug, Deserialize)]
pub struct ServerStateResult {
    /// Detailed server state information.
    pub state: ServerState,
}

/// Machine-readable server state details returned inside a `ServerStateResult`.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ServerState {
    /// Whether the server is blocked from participating due to unsupported amendments.
    pub amendment_blocked: Option<bool>,
    /// rippled build version string.
    pub build_version: String,
    /// Range(s) of ledger versions the server has locally, e.g. "63000000-63500000".
    pub complete_ledgers: String,
    /// Median I/O latency in milliseconds; high values indicate disk pressure.
    pub io_latency_ms: u64,
    /// Count of transactions dropped due to job queue overflow.
    pub jq_trans_overflow: String,
    /// Timing details from the most recent ledger close.
    pub last_close: LastClose,
    /// Reference load level (always 256).
    pub load_base: u64,
    /// Current load factor applied to the base transaction cost.
    pub load_factor: u64,
    /// Load factor from fee escalation for the open ledger.
    pub load_factor_fee_escalation: u64,
    /// Load factor applied to transactions held in the fee queue.
    pub load_factor_fee_queue: u64,
    /// Fee reference load factor (usually 256).
    pub load_factor_fee_reference: u64,
    /// Load factor from server-side resource constraints.
    pub load_factor_server: u64,
    /// Network ID distinguishing mainnet from sidechains or testnets.
    pub network_id: Option<u64>,
    /// Total number of peer disconnects since startup.
    pub peer_disconnects: String,
    /// Peer disconnects caused by resource exhaustion.
    pub peer_disconnects_resources: String,
    /// Number of currently connected peers.
    pub peers: u64,
    /// Ed25519 public key identifying this node in the peer network.
    pub pubkey_node: String,
    /// Current server state, e.g. "full", "syncing", "connected".
    pub server_state: String,
    /// Time spent in the current server state, in microseconds.
    pub server_state_duration_us: String,
    /// Per-state duration and transition counters since startup.
    pub state_accounting: StateAccounting,
    /// Current UTC time on the server.
    pub time: String,
    /// Server uptime in seconds.
    pub uptime: u64,
    /// Most recently validated ledger summary.
    pub validated_ledger: Option<ValidatedLedger>,
    /// Minimum number of trusted validator votes required to validate a ledger.
    pub validation_quorum: u64,
    /// Ports and protocols this server is listening on.
    pub ports: Option<Vec<ServerStatePort>>,
}

/// A port descriptor for a server_state response.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ServerStatePort {
    /// Port number string.
    pub port: String,
    /// Protocols served on this port.
    pub protocol: Vec<String>,
}

/// Timing information from the most recent ledger close (server_state variant).
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct LastClose {
    /// Time the consensus round took to converge, in milliseconds.
    pub converge_time: u64,
    /// Number of trusted validators that participated in the consensus round.
    pub proposers: u64,
}

/// Per-state duration counters for a `server_state` response.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct StateAccounting {
    /// Time and transitions spent in the "connected" state.
    pub connected: AccountingState,
    /// Time and transitions spent in the "disconnected" state.
    pub disconnected: AccountingState,
    /// Time and transitions spent in the "full" (synced) state.
    pub full: AccountingState,
    /// Time and transitions spent in the "syncing" state.
    pub syncing: AccountingState,
    /// Time and transitions spent in the "tracking" state.
    pub tracking: AccountingState,
}

/// Duration and transition count for a single server state.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct AccountingState {
    /// Total time spent in this state since startup, in microseconds.
    pub duration_us: String,
    /// Number of times the server entered this state.
    pub transitions: String,
}

/// Summary of the most recently validated ledger from `server_state` (values in drops).
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ValidatedLedger {
    /// Seconds since this ledger was validated.
    pub age: u64,
    /// Reference transaction cost in drops.
    pub base_fee: u64,
    /// Close time as Ripple epoch seconds.
    pub close_time: u64,
    /// Hash of the most recently validated ledger.
    pub hash: String,
    /// Base account reserve in drops.
    pub reserve_base: u64,
    /// Owner reserve increment per object in drops.
    pub reserve_inc: u64,
    /// Sequence number of the most recently validated ledger.
    pub seq: u64,
}
