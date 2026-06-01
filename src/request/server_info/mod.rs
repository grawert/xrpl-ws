use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{XrplRequest, XrplResponse};

/// Retrieves a human-readable summary of the server's state and ledger chain.
///
/// Useful for health checks, build version detection, and monitoring sync status.
///
/// # Example
/// ```rust
/// use xrpl::request::server_info::ServerInfoRequest;
///
/// let request = ServerInfoRequest {
///     counters: Some(true),
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct ServerInfoRequest {
    /// If `true`, return metrics about the job queue, ledger store, and API method activity.
    pub counters: Option<bool>,
}

impl XrplRequest for ServerInfoRequest {
    type Response = XrplResponse<ServerInfoResult>;
    const COMMAND: &str = "server_info";
}

/// Response to a `server_info` request.
#[derive(Clone, Debug, Deserialize)]
pub struct ServerInfoResult {
    /// Detailed server state information.
    pub info: ServerInfo,
}

/// Server state details returned inside a `ServerInfoResult`.
#[derive(Clone, Debug, Deserialize)]
pub struct ServerInfo {
    /// Whether the server is blocked from participating due to unsupported amendments.
    pub amendment_blocked: Option<bool>,
    /// rippled build version string.
    pub build_version: String,
    /// URL of the Clio server this server is connected to.
    pub clio_server_url: Option<String>,
    /// Most recently closed ledger, when the server is not yet synced to validated.
    pub closed_ledger: Option<serde_json::Value>,
    /// Range(s) of ledger versions the server has locally, e.g. "63000000-63500000".
    pub complete_ledgers: String,
    /// Human-readable hostname identifier for this node.
    pub hostid: String,
    /// Median I/O latency in milliseconds; high values indicate disk pressure.
    pub io_latency_ms: u64,
    /// Count of transactions dropped due to job queue overflow.
    pub jq_trans_overflow: String,
    /// Timing details from the most recent ledger close.
    pub last_close: ServerInfoLastClose,
    /// Current load factor relative to the base transaction cost.
    pub load_factor: f64,
    /// Current multiplier to the transaction cost to get into the open ledger.
    pub load_factor_fee_escalation: Option<f64>,
    /// Current multiplier to the transaction cost to get into the queue.
    pub load_factor_fee_queue: Option<f64>,
    /// The load factor being used as a reference for fee calculation.
    pub load_factor_fee_reference: Option<f64>,
    /// Current multiplier to the transaction cost based on load to the server.
    pub load_factor_server: Option<f64>,
    /// Network ID distinguishing mainnet from sidechains or testnets.
    pub network_id: Option<u64>,
    /// Total number of peer disconnects since startup.
    pub peer_disconnects: String,
    /// Peer disconnects caused by resource exhaustion.
    pub peer_disconnects_resources: String,
    /// Number of currently connected peers.
    pub peers: u64,
    /// Ports and protocols this server is listening on.
    pub ports: Option<Vec<ServerInfoPort>>,
    /// Ed25519 public key identifying this node in the peer network.
    pub pubkey_node: String,
    /// Information about the reporting mode server.
    pub reporting: Option<ServerInfoReporting>,
    /// Current server state, e.g. "full", "syncing", "connected".
    pub server_state: String,
    /// Time spent in the current server state, in microseconds.
    pub server_state_duration_us: String,
    /// Per-state duration and transition counters since startup.
    pub state_accounting: ServerInfoStateAccounting,
    /// Current UTC time on the server.
    pub time: String,
    /// Server uptime in seconds.
    pub uptime: u64,
    /// Most recently validated ledger summary; absent while syncing.
    pub validated_ledger: Option<ServerInfoValidatedLedger>,
    /// Minimum number of trusted validator votes required to validate a ledger.
    pub validation_quorum: u64,
}

/// Reporting mode server details.
#[derive(Clone, Debug, Deserialize)]
pub struct ServerInfoReporting {
    /// Whether this server is a writer.
    pub is_writer: bool,
    /// The URL of the Clio server this server is connected to.
    pub clio_server_url: Option<String>,
    /// Information about ETL sources.
    pub etl_sources: Option<Vec<ServerInfoEtlSource>>,
}

/// ETL source details.
#[derive(Clone, Debug, Deserialize)]
pub struct ServerInfoEtlSource {
    /// The IP address of the ETL source.
    pub ip: String,
    /// The port of the ETL source.
    pub port: u32,
    /// The protocol specification.
    pub spec: String,
    /// Whether the source is validated.
    pub validated: bool,
}

/// A port descriptor for a server_info response.
#[derive(Clone, Debug, Deserialize)]
pub struct ServerInfoPort {
    /// Port number string.
    pub port: String,
    /// Protocols served on this port.
    pub protocol: Vec<String>,
}

/// Summary of the most recently validated ledger from `server_info`.
#[derive(Clone, Debug, Deserialize)]
pub struct ServerInfoValidatedLedger {
    /// Seconds since this ledger was validated.
    pub age: u64,
    /// Reference transaction cost in XRP (not drops).
    pub base_fee_xrp: f64,
    /// Hash of the most recently validated ledger.
    pub hash: String,
    /// Base account reserve in XRP.
    pub reserve_base_xrp: f64,
    /// Owner reserve increment per object in XRP.
    pub reserve_inc_xrp: f64,
    /// Sequence number of the most recently validated ledger.
    pub seq: u32,
}

/// Timing information from the most recent ledger close.
#[derive(Clone, Debug, Deserialize)]
pub struct ServerInfoLastClose {
    /// Time the consensus round took to converge, in seconds.
    pub converge_time_s: f64,
    /// Number of trusted validators that participated in the consensus round.
    pub proposers: u32,
}

/// Per-state duration counters for a `server_info` response.
#[derive(Clone, Debug, Deserialize)]
pub struct ServerInfoStateAccounting {
    /// Time and transitions spent in the "connected" state.
    pub connected: ServerInfoStateAccount,
    /// Time and transitions spent in the "disconnected" state.
    pub disconnected: ServerInfoStateAccount,
    /// Time and transitions spent in the "full" (synced) state.
    pub full: ServerInfoStateAccount,
    /// Time and transitions spent in the "syncing" state.
    pub syncing: ServerInfoStateAccount,
    /// Time and transitions spent in the "tracking" state.
    pub tracking: ServerInfoStateAccount,
}

/// Duration and transition count for a single server state.
#[derive(Clone, Debug, Deserialize)]
pub struct ServerInfoStateAccount {
    /// Total time spent in this state since startup, in microseconds.
    pub duration_us: String,
    /// Number of times the server entered this state.
    pub transitions: String,
}
