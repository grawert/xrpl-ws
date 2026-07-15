use serde::{Deserialize, Serialize};

use super::{XrplRequest, XrplResponse};

/// Retrieves the current transaction cost levels from the server.
///
/// Useful for choosing an appropriate fee before submitting a transaction.
///
/// # Example
/// ```rust
/// use xrpl::request::fee::FeeRequest;
///
/// let request = FeeRequest;
/// ```
#[derive(Debug, Clone, Default, Serialize)]
pub struct FeeRequest;

impl XrplRequest for FeeRequest {
    type Response = XrplResponse<FeeResult>;
    const COMMAND: &str = "fee";
}

/// Response to a `fee` request.
#[derive(Clone, Debug, Deserialize)]
pub struct FeeResult {
    /// Number of transactions provisionally included in the in-progress ledger.
    pub current_ledger_size: String,
    /// Number of transactions currently queued for the next ledger.
    pub current_queue_size: String,
    /// Fee levels expressed in drops of XRP.
    pub drops: FeeDrops,
    /// The approximate number of transactions expected to be included in the current ledger.
    pub expected_ledger_size: String,
    /// Sequence number of the current open ledger.
    pub ledger_current_index: u32,
    /// Fee levels expressed in abstract fee units (useful for relative comparisons).
    pub levels: FeeLevels,
    /// The maximum number of transactions that the transaction queue can currently hold.
    pub max_queue_size: String,
}

/// Transaction cost thresholds in drops of XRP.
#[derive(Clone, Debug, Deserialize)]
pub struct FeeDrops {
    /// Cost of a reference transaction at normal load.
    pub base_fee: String,
    /// Median fee among recently validated transactions.
    pub median_fee: String,
    /// Minimum fee that will be accepted by the node into its queue.
    pub minimum_fee: String,
    /// Minimum fee to be included in the current open ledger immediately.
    pub open_ledger_fee: String,
}

/// Transaction cost thresholds expressed in abstract fee units (1 unit = base_fee / 256).
#[derive(Clone, Debug, Deserialize)]
pub struct FeeLevels {
    /// Median fee level among recently validated transactions.
    pub median_level: String,
    /// Minimum fee level accepted into the node's queue.
    pub minimum_level: String,
    /// Minimum fee level to enter the current open ledger immediately.
    pub open_ledger_level: String,
    /// The equivalent of the minimum transaction cost, represented in fee levels.
    pub reference_level: String,
}
