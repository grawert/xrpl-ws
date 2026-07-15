use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Removes an on-ledger price oracle document.
///
/// Only the oracle's owner account can delete it. Once deleted, the document ID
/// can be reused.
///
/// ```rust
/// use xrpl::types::transactions::oracle::OracleDelete;
/// let tx = OracleDelete { oracle_document_id: 1 };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct OracleDelete {
    /// Unique identifier of the oracle document to remove.
    #[serde(rename = "OracleDocumentID")]
    pub oracle_document_id: u32,
}

/// Creates or updates an on-ledger price oracle with a series of price data entries.
///
/// Each entry in `price_data_series` represents a base/quote asset pair. A new
/// `oracle_document_id` creates the oracle; an existing ID updates it.
///
/// ```rust
/// use xrpl::types::transactions::oracle::{OracleSet, PriceDataWrapper, PriceData};
/// let tx = OracleSet {
///     oracle_document_id: 1,
///     last_update_time: 946_684_800,
///     price_data_series: vec![],
///     asset_class: Some("currency".to_string()),
///     provider: None,
///     uri: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct OracleSet {
    /// Hex-encoded string describing the asset class (e.g. `"currency"`).
    #[serde(rename = "AssetClass")]
    pub asset_class: Option<String>,
    /// Timestamp of the most recent price update (seconds since the Ripple epoch).
    #[serde(rename = "LastUpdateTime")]
    pub last_update_time: u32,
    /// Unique identifier for this oracle document (created or updated).
    #[serde(rename = "OracleDocumentID")]
    pub oracle_document_id: u32,
    /// One or more base/quote price entries.
    #[serde(rename = "PriceDataSeries")]
    pub price_data_series: Vec<PriceDataWrapper>,
    /// Hex-encoded name or identifier of the price data provider.
    #[serde(rename = "Provider")]
    pub provider: Option<String>,
    /// Hex-encoded URI pointing to additional oracle metadata.
    #[serde(rename = "URI")]
    pub uri: Option<String>,
}

/// Wire-format wrapper that nests `PriceData` under the `PriceData` key.
///
/// Required by the XRPL JSON protocol; use [`PriceData`] for the actual price entry.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PriceDataWrapper {
    /// The price data entry.
    pub price_data: PriceData,
}

impl From<PriceData> for PriceDataWrapper {
    fn from(price_data: PriceData) -> Self {
        Self { price_data }
    }
}

/// One base/quote price entry within an oracle's `PriceDataSeries`.
///
/// The raw integer price is stored in `asset_price` (as a decimal string) and
/// scaled by `10^-scale` to get the actual value.
///
/// ```rust
/// use xrpl::types::transactions::oracle::PriceData;
/// let entry = PriceData {
///     base_asset: "XRP".to_string(),
///     quote_asset: "USD".to_string(),
///     asset_price: Some("5000".to_string()), // 0.50 USD with scale=4
///     scale: Some(4),
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PriceData {
    /// Raw price as a UInt64 decimal string (actual value = asset_price x 10^-scale).
    pub asset_price: Option<String>,
    /// Currency code or asset symbol for the base asset.
    pub base_asset: String,
    /// Currency code or asset symbol for the quote asset.
    pub quote_asset: String,
    /// Number of decimal places to apply to `asset_price` (0-10).
    pub scale: Option<u8>,
}

impl PriceData {
    /// Creates a `PriceData` entry with only the required base/quote assets.
    /// Use [`with_price`] to attach a quoted price and scale.
    ///
    /// [`with_price`]: PriceData::with_price
    pub fn new(
        base_asset: impl AsRef<str>,
        quote_asset: impl AsRef<str>,
    ) -> Self {
        Self {
            asset_price: None,
            base_asset: base_asset.as_ref().to_string(),
            quote_asset: quote_asset.as_ref().to_string(),
            scale: None,
        }
    }

    /// Attaches a raw price and the decimal scale to apply to it.
    pub fn with_price(
        mut self,
        asset_price: impl AsRef<str>,
        scale: u8,
    ) -> Self {
        self.asset_price = Some(asset_price.as_ref().to_string());
        self.scale = Some(scale);
        self
    }
}
