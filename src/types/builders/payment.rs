use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::{validate_address, validate_amount},
    Amount, PathStep, TransactionType,
};

/// Builder for XRPL payment transactions.
///
/// # Example
/// ```no_run
/// use xrpl::{xrp, drops};
/// use xrpl::types::{Amount, Memo, builders::PaymentBuilder};
/// use anyhow::Result;
/// fn main() -> Result<()> {
///     let memo = Memo {
///         memo_data: Some("72656e74".to_string()),
///         memo_type: Some("746578742f706c61696e".to_string()),
///         memo_format: None,
///     };
///     let payment = PaymentBuilder::new(
///         "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
///         1,
///         drops!(10),
///         xrp!(1.99),
///     )
///     .with_destination_tag(12345)
///     .with_memos(vec![memo])
///     .build()?;
///     Ok(())
/// }
/// ```
pub struct Payment {
    pub destination: String,
    pub deliver_max: Amount,
    pub deliver_min: Option<Amount>,
    pub destination_tag: Option<u32>,
    pub invoice_id: Option<String>,
    pub paths: Option<Vec<Vec<PathStep>>>,
    pub send_max: Option<Amount>,
}

pub type PaymentBuilder = TransactionBuilder<Payment>;

impl PaymentBuilder {
    pub fn new(
        account: String,
        destination: String,
        sequence: u32,
        fee: Amount,
        deliver_max: Amount,
    ) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            Payment {
                destination,
                deliver_max,
                deliver_min: None,
                destination_tag: None,
                invoice_id: None,
                paths: None,
                send_max: None,
            },
        )
    }

    pub fn with_destination_tag(mut self, tag: u32) -> Self {
        self.transaction_type.destination_tag = Some(tag);
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

    pub fn with_deliver_min(mut self, amount: Amount) -> Self {
        self.transaction_type.deliver_min = Some(amount);
        self
    }

    pub fn with_send_max(mut self, amount: Amount) -> Self {
        self.transaction_type.send_max = Some(amount);
        self
    }

    pub fn add_path(mut self, path: Vec<PathStep>) -> Self {
        self.transaction_type.paths.get_or_insert_with(Vec::new).push(path);
        self
    }
}

impl TransactionTypeBuilder for Payment {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        validate_amount(&self.deliver_max)?;
        validate_address(&self.destination)?;
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        Ok(TransactionType::Payment {
            amount: Some(self.deliver_max.clone()),
            deliver_max: Some(self.deliver_max),
            deliver_min: self.deliver_min,
            destination: self.destination,
            destination_tag: self.destination_tag,
            invoice_id: self.invoice_id,
            paths: self.paths,
            send_max: self.send_max,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Memo;

    const SEQUENCE: u32 = 1;

    #[test]
    fn test_payment_builder_basic() {
        let payment = PaymentBuilder::new(
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
            "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
            SEQUENCE,
            drops!(10),
            xrp!(1),
        )
        .build()
        .expect("Should build valid payment");

        assert_eq!(payment.sequence, 1);
        assert_eq!(payment.fee, "10");

        if let TransactionType::Payment {
            destination,
            amount,
            deliver_max,
            ..
        } = payment.transaction_type
        {
            assert_eq!(destination, "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
            assert_eq!(amount, deliver_max);
            assert_eq!(amount, Some(Amount::Xrpl("1000000".to_string())));
        } else {
            panic!("Expected Payment transaction type");
        }
    }

    #[test]
    fn test_payment_builder_with_memo() {
        let memo = Memo {
            memo_data: Some("48656c6c6f".to_string()),
            memo_format: None,
            memo_type: None,
        };

        let payment = PaymentBuilder::new(
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
            "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
            SEQUENCE,
            drops!(10),
            xrp!(1),
        )
        .with_memos(vec![memo])
        .build()
        .expect("Should build valid payment");

        assert_eq!(payment.memos.unwrap().len(), 1);
    }

    #[test]
    fn test_payment_builder_with_destination_tag() {
        let payment = PaymentBuilder::new(
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
            "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
            SEQUENCE,
            drops!(10),
            xrp!(1),
        )
        .with_destination_tag(12345)
        .build()
        .expect("Should build valid payment");

        if let TransactionType::Payment { destination_tag, .. } =
            payment.transaction_type
        {
            assert_eq!(destination_tag, Some(12345));
        } else {
            panic!("Expected Payment transaction type");
        }
    }

    #[test]
    fn test_payment_builder_invalid_account() {
        let result = PaymentBuilder::new(
            "not_an_address".to_string(),
            "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
            SEQUENCE,
            drops!(10),
            xrp!(1),
        )
        .build();

        assert!(matches!(result, Err(BuildError::Validation(_))));
    }

    #[test]
    fn test_payment_builder_invalid_destination() {
        let result = PaymentBuilder::new(
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
            "not_an_address".to_string(),
            SEQUENCE,
            drops!(10),
            xrp!(1),
        )
        .build();

        assert!(matches!(result, Err(BuildError::Validation(_))));
    }

    #[test]
    fn test_payment_builder_with_issued_currency() {
        let payment = PaymentBuilder::new(
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
            "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
            SEQUENCE,
            drops!(10),
            Amount::IssuedCurrency {
                value: "100.50".to_string(),
                currency: "USD".to_string(),
                issuer: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
            },
        )
        .build()
        .expect("Should build valid payment with issued currency");

        if let TransactionType::Payment { amount, .. } =
            payment.transaction_type
        {
            if let Some(Amount::IssuedCurrency { value, currency, issuer }) =
                amount
            {
                assert_eq!(value, "100.50");
                assert_eq!(currency, "USD");
                assert_eq!(issuer, "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
            } else {
                panic!("Expected IssuedCurrency amount");
            }
        } else {
            panic!("Expected Payment transaction type");
        }
    }

    #[test]
    fn test_payment_builder_with_invoice_id() {
        use sha2::{Sha256, Digest};

        let invoice_id = hex::encode(Sha256::digest("invoice-2026-001"));

        let payment = PaymentBuilder::new(
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
            "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
            SEQUENCE,
            drops!(10),
            xrp!(1),
        )
        .with_invoice_id(&invoice_id)
        .build()
        .expect("Should build valid payment with invoice id");

        if let TransactionType::Payment { invoice_id: id, .. } =
            payment.transaction_type
        {
            assert_eq!(id, Some(invoice_id));
        } else {
            panic!("Expected Payment transaction type");
        }
    }
}
