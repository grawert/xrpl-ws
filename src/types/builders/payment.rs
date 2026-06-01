use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::{validate_address, validate_amount, validate_invoice_id},
    transactions::payment::{PathStep, Payment},
    Amount, TransactionType,
};

/// Builder for XRPL payment transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, xrp, types::{Memo, builders::PaymentBuilder}};
/// let client = Client::new("wss://xrplcluster.com");
/// let memo = Memo {
///     memo_data: Some("72656e74".to_string()),
///     memo_type: Some("746578742f706c61696e".to_string()),
///     memo_format: None,
/// };
/// let tx = PaymentBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     xrp!(1.99),
/// )
/// .with_destination_tag(12345)
/// .with_memos(vec![memo])
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type PaymentBuilder = TransactionBuilder<Payment>;

impl PaymentBuilder {
    /// Creates a new `PaymentBuilder` with the required sender, destination, and amount.
    ///
    /// For cross-currency payments, also call [`with_send_max`] to set the spending cap.
    /// Call [`fill`] to populate `sequence`, `fee`, and `last_ledger_sequence` from the network.
    ///
    /// [`with_send_max`]: PaymentBuilder::with_send_max
    /// [`fill`]: crate::types::builders::TransactionBuilder::fill
    pub fn new(
        account: impl Into<String>,
        destination: impl Into<String>,
        amount: impl Into<Amount>,
    ) -> Self {
        let amount = amount.into();
        Self::init(
            account,
            0,
            Amount::default(),
            Payment {
                deliver_max: Some(amount.clone()),
                amount: Some(amount),
                destination: destination.into(),
                deliver_min: None,
                destination_tag: None,
                invoice_id: None,
                paths: None,
                send_max: None,
            },
        )
    }

    /// Sets the destination tag for routing within the recipient account.
    pub fn with_destination_tag(mut self, tag: u32) -> Self {
        self.transaction_type.destination_tag = Some(tag);
        self
    }

    /// Sets the 64-character hex invoice ID for reconciliation.
    pub fn with_invoice_id(
        mut self,
        invoice_id: impl Into<String>,
    ) -> Result<Self, BuildError> {
        let id = invoice_id.into();
        validate_invoice_id(&id)?;
        self.transaction_type.invoice_id = Some(id);
        Ok(self)
    }

    /// Sets the minimum amount to deliver when [`PaymentFlag::PartialPayment`] is active.
    ///
    /// [`PaymentFlag::PartialPayment`]: crate::types::PaymentFlag::PartialPayment
    pub fn with_deliver_min(mut self, amount: impl Into<Amount>) -> Self {
        self.transaction_type.deliver_min = Some(amount.into());
        self
    }

    /// Sets the maximum amount the sender is willing to spend for cross-currency payments.
    pub fn with_send_max(mut self, amount: impl Into<Amount>) -> Self {
        self.transaction_type.send_max = Some(amount.into());
        self
    }

    /// Appends a payment path (sequence of hops) for cross-currency routing.
    ///
    /// Accepts any iterable of items convertible into [`PathStep`].
    pub fn add_path<I, P>(mut self, path: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathStep>,
    {
        let steps: Vec<PathStep> = path.into_iter().map(Into::into).collect();
        self.transaction_type.paths.get_or_insert_with(Vec::new).push(steps);
        self
    }
}

impl TransactionTypeBuilder for Payment {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(amount) = &self.amount {
            validate_amount(amount)?;
        }
        validate_address(&self.destination)?;
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::Payment(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Memo;

    const SENDER: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
    const RECEIVER: &str = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";

    #[test]
    fn test_payment_builder_basic() {
        let payment = PaymentBuilder::new(SENDER, RECEIVER, xrp!(1))
            .build()
            .expect("Should build valid payment");

        if let TransactionType::Payment(Payment {
            destination,
            amount,
            deliver_max,
            ..
        }) = payment.transaction_type
        {
            assert_eq!(destination, RECEIVER);
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

        let payment = PaymentBuilder::new(SENDER, RECEIVER, xrp!(1))
            .with_memos(vec![memo])
            .build()
            .expect("Should build valid payment");

        assert_eq!(payment.memos.unwrap().len(), 1);
    }

    #[test]
    fn test_payment_builder_with_destination_tag() {
        let payment = PaymentBuilder::new(SENDER, RECEIVER, xrp!(1))
            .with_destination_tag(12345)
            .build()
            .expect("Should build valid payment");

        if let TransactionType::Payment(Payment { destination_tag, .. }) =
            payment.transaction_type
        {
            assert_eq!(destination_tag, Some(12345));
        } else {
            panic!("Expected Payment transaction type");
        }
    }

    #[test]
    fn test_payment_builder_invalid_account() {
        let result =
            PaymentBuilder::new("not_an_address", RECEIVER, xrp!(1)).build();

        assert!(matches!(result, Err(BuildError::Validation(_))));
    }

    #[test]
    fn test_payment_builder_invalid_destination() {
        let result =
            PaymentBuilder::new(SENDER, "not_an_address", xrp!(1)).build();

        assert!(matches!(result, Err(BuildError::Validation(_))));
    }

    #[test]
    fn test_payment_builder_with_issued_currency() {
        let payment = PaymentBuilder::new(
            SENDER,
            RECEIVER,
            Amount::IssuedCurrency {
                value: "100.50".to_string(),
                currency: "USD".to_string(),
                issuer: SENDER.to_string(),
            },
        )
        .build()
        .expect("Should build valid payment with issued currency");

        if let TransactionType::Payment(Payment { amount, .. }) =
            payment.transaction_type
        {
            if let Some(Amount::IssuedCurrency { value, currency, issuer }) =
                amount
            {
                assert_eq!(value, "100.50");
                assert_eq!(currency, "USD");
                assert_eq!(issuer, SENDER);
            } else {
                panic!("Expected IssuedCurrency amount");
            }
        } else {
            panic!("Expected Payment transaction type");
        }
    }

    #[test]
    fn test_payment_builder_with_invoice_id() {
        use sha2::{Digest, Sha256};

        let invoice_id = hex::encode(Sha256::digest("invoice-2026-001"));

        let payment = PaymentBuilder::new(SENDER, RECEIVER, xrp!(1))
            .with_invoice_id(&invoice_id)
            .expect("Should accept valid invoice id")
            .build()
            .expect("Should build valid payment with invoice id");

        if let TransactionType::Payment(Payment { invoice_id: id, .. }) =
            payment.transaction_type
        {
            assert_eq!(id.as_deref(), Some(invoice_id.as_str()));
        } else {
            panic!("Expected Payment transaction type");
        }
    }

    #[test]
    fn test_payment_builder_invalid_invoice_id() {
        let result = PaymentBuilder::new(SENDER, RECEIVER, xrp!(1))
            .with_invoice_id("not-a-valid-hex-id");

        assert!(matches!(result, Err(BuildError::Validation(_))));
    }
}
