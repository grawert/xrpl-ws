use serde::{Deserialize, Serialize};
use super::Asset;

/// Identifies the two door accounts and assets of a cross-chain bridge.
///
/// # Example
/// ```rust
/// use xrpl::types::{Asset, xchain::XChainBridge};
///
/// let bridge = XChainBridge {
///     locking_chain_door: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///     locking_chain_issue: Asset::xrp(),
///     issuing_chain_door: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
///     issuing_chain_issue: Asset::xrp(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XChainBridge {
    /// r-address of the door account on the locking chain.
    pub locking_chain_door: String,
    /// Asset locked on the locking chain (XRP or issued currency).
    pub locking_chain_issue: Asset,
    /// r-address of the door account on the issuing chain.
    pub issuing_chain_door: String,
    /// Wrapped asset minted on the issuing chain.
    pub issuing_chain_issue: Asset,
}

impl XChainBridge {
    /// Creates a new `XChainBridge` describing the two chains and their assets.
    pub fn new(
        locking_chain_door: impl Into<String>,
        locking_chain_issue: impl Into<Asset>,
        issuing_chain_door: impl Into<String>,
        issuing_chain_issue: impl Into<Asset>,
    ) -> Self {
        Self {
            locking_chain_door: locking_chain_door.into(),
            locking_chain_issue: locking_chain_issue.into(),
            issuing_chain_door: issuing_chain_door.into(),
            issuing_chain_issue: issuing_chain_issue.into(),
        }
    }
}
