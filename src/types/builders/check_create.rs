use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::{validate_address, validate_amount},
    Amount, TransactionType,
};

/// Builder for XRPL CheckCreate transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{xrp, drops};
/// use xrpl::types::{Amount, builders::CheckCreateBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let check_create = CheckCreateBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
///         1,
///         drops!(10),
///         xrp!(100),
///     )
///     .with_destination_tag(12345)
///     .with_expiration(1234567890)
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct CheckCreate {
    pub destination: String,
    pub send_max: Amount,
    pub destination_tag: Option<u32>,
    pub expiration: Option<u32>,
    pub invoice_id: Option<String>,
}

pub type CheckCreateBuilder = TransactionBuilder<CheckCreate>;

impl CheckCreateBuilder {
    pub fn new(
        account: String,
        destination: String,
        sequence: u32,
        fee: Amount,
        send_max: Amount,
    ) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            CheckCreate {
                destination,
                send_max,
                destination_tag: None,
                expiration: None,
                invoice_id: None,
            },
        )
    }

    pub fn with_destination_tag(mut self, tag: u32) -> Self {
        self.transaction_type.destination_tag = Some(tag);
        self
    }

    pub fn with_expiration(mut self, expiration: u32) -> Self {
        self.transaction_type.expiration = Some(expiration);
        self
    }

    pub fn with_invoice_id(mut self, invoice_id: impl Into<String>) -> Self {
        let id = invoice_id.into();
        if id.len() != 64 || !id.chars().all(|c| c.is_ascii_hexdigit()) {
            panic!("InvoiceID must be a 64-character hex string (32 bytes)");
        }
        self.transaction_type.invoice_id = Some(id);
        self
    }
}

impl TransactionTypeBuilder for CheckCreate {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_address(&self.destination)?;
        validate_amount(&self.send_max)?;
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::CheckCreate {
            destination: self.destination,
            send_max: self.send_max,
            destination_tag: self.destination_tag,
            expiration: self.expiration,
            invoice_id: self.invoice_id,
        })
    }
}
