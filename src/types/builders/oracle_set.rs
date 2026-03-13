use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{transaction::PriceDataWrapper, Amount, TransactionType};

/// Builder for XRPL OracleSet transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{drops};
/// use xrpl::types::{Amount, builders::OracleSetBuilder, transaction::PriceDataWrapper};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let oracle_set = OracleSetBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         1,
///         drops!(10),
///         1234567890,
///         123,
///         vec![], // Empty price data series for example
///     )
///     .with_asset_class("currency".to_string())
///     .with_provider("example-provider".to_string())
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct OracleSet {
    pub asset_class: Option<String>,
    pub last_update_time: u32,
    pub oracle_document_id: u32,
    pub price_data_series: Vec<PriceDataWrapper>,
    pub provider: Option<String>,
    pub uri: Option<String>,
}

pub type OracleSetBuilder = TransactionBuilder<OracleSet>;

impl OracleSetBuilder {
    pub fn new(
        account: String,
        sequence: u32,
        fee: Amount,
        last_update_time: u32,
        oracle_document_id: u32,
        price_data_series: Vec<PriceDataWrapper>,
    ) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            OracleSet {
                asset_class: None,
                last_update_time,
                oracle_document_id,
                price_data_series,
                provider: None,
                uri: None,
            },
        )
    }

    pub fn with_asset_class(mut self, asset_class: String) -> Self {
        self.transaction_type.asset_class = Some(asset_class);
        self
    }

    pub fn with_provider(mut self, provider: String) -> Self {
        self.transaction_type.provider = Some(provider);
        self
    }

    pub fn with_uri(mut self, uri: String) -> Self {
        self.transaction_type.uri = Some(uri);
        self
    }

    pub fn add_price_data(mut self, price_data: PriceDataWrapper) -> Self {
        self.transaction_type.price_data_series.push(price_data);
        self
    }
}

impl TransactionTypeBuilder for OracleSet {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        // Oracle-specific validation could go here
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::OracleSet {
            asset_class: self.asset_class,
            last_update_time: self.last_update_time,
            oracle_document_id: self.oracle_document_id,
            price_data_series: self.price_data_series,
            provider: self.provider,
            uri: self.uri,
        })
    }
}
