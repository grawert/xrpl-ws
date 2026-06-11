use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    transactions::oracle::{OracleSet, PriceDataWrapper},
    Amount, TransactionType,
};

/// Builder for XRPL OracleSet transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::{builders::OracleSetBuilder, transactions::oracle::PriceDataWrapper}};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = OracleSetBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     960000000,
///     1,
///     Vec::<PriceDataWrapper>::new(),
/// )
/// .with_asset_class("63757272656e6379") // hex-encoded "currency"
/// .with_provider("70726f7669646572") // hex-encoded "provider"
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type OracleSetBuilder = TransactionBuilder<OracleSet>;

impl OracleSetBuilder {
    /// Creates a new `OracleSetBuilder` with the required fields.
    pub fn new(
        account: impl AsRef<str>,
        last_update_time: u32,
        oracle_document_id: u32,
        price_data_series: impl IntoIterator<Item = impl Into<PriceDataWrapper>>,
    ) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            OracleSet {
                asset_class: None,
                last_update_time,
                oracle_document_id,
                price_data_series: price_data_series
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                provider: None,
                uri: None,
            },
        )
    }

    /// Sets the hex-encoded asset class string (e.g. `"currency"`).
    pub fn with_asset_class(mut self, asset_class: impl AsRef<str>) -> Self {
        self.transaction_type.asset_class =
            Some(asset_class.as_ref().to_string());
        self
    }

    /// Sets the hex-encoded name of the price data provider.
    pub fn with_provider(mut self, provider: impl AsRef<str>) -> Self {
        self.transaction_type.provider = Some(provider.as_ref().to_string());
        self
    }

    /// Sets the hex-encoded URI pointing to additional oracle metadata.
    pub fn with_uri(mut self, uri: impl AsRef<str>) -> Self {
        self.transaction_type.uri = Some(uri.as_ref().to_string());
        self
    }

    /// Appends a price data entry to the series.
    pub fn add_price_data(
        mut self,
        price_data: impl Into<PriceDataWrapper>,
    ) -> Self {
        self.transaction_type.price_data_series.push(price_data.into());
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
        Ok(TransactionType::OracleSet(self))
    }
}
