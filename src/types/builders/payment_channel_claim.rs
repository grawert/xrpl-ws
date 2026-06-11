use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::validate_amount,
    transactions::payment_channel::PaymentChannelClaim, Amount,
    TransactionType,
};

/// Builder for XRPL PaymentChannelClaim transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # use xrpl::types::Amount;
/// use xrpl::{Client, xrp, types::builders::PaymentChannelClaimBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = PaymentChannelClaimBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "5DB01B7FFED6B67E6B0414DED11E051D2EE2B7619CE0EAA6286D67A3A4D5BDB3",
/// )
/// .with_amount(xrp!(50))
/// .fill(&client)
/// .await?
/// .build()?;
/// # Ok(())
/// # }
/// ```
pub type PaymentChannelClaimBuilder = TransactionBuilder<PaymentChannelClaim>;

impl PaymentChannelClaimBuilder {
    /// Creates a new `PaymentChannelClaimBuilder` for the specified channel.
    pub fn new(account: impl AsRef<str>, channel: impl AsRef<str>) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            PaymentChannelClaim {
                channel: channel.as_ref().to_string(),
                amount: None,
                balance: None,
                credential_ids: None,
                public_key: None,
                signature: None,
            },
        )
    }

    /// Sets the total XRP (drops) the channel can pay out after this claim.
    pub fn with_amount(mut self, amount: impl Into<Amount>) -> Self {
        self.transaction_type.amount = Some(amount.into());
        self
    }

    /// Sets the cumulative XRP (drops) delivered by the channel so far.
    pub fn with_balance(mut self, balance: impl Into<Amount>) -> Self {
        self.transaction_type.balance = Some(balance.into());
        self
    }

    /// Sets the credential IDs used to satisfy deposit authorization on the destination.
    pub fn with_credential_ids(
        mut self,
        credential_ids: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Self {
        self.transaction_type.credential_ids = Some(
            credential_ids
                .into_iter()
                .map(|s| s.as_ref().to_string())
                .collect(),
        );
        self
    }

    /// Sets the sender's public key used to verify the claim signature.
    pub fn with_public_key(mut self, public_key: impl AsRef<str>) -> Self {
        self.transaction_type.public_key =
            Some(public_key.as_ref().to_string());
        self
    }

    /// Sets the sender's signature authorizing the claim amount.
    pub fn with_signature(mut self, signature: impl AsRef<str>) -> Self {
        self.transaction_type.signature = Some(signature.as_ref().to_string());
        self
    }
}

impl TransactionTypeBuilder for PaymentChannelClaim {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(amount) = &self.amount {
            validate_amount(amount)?;
        }
        if let Some(balance) = &self.balance {
            validate_amount(balance)?;
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::PaymentChannelClaim(self))
    }
}
