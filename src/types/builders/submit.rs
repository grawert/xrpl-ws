use crate::request::submit::SubmitRequest;
use crate::request::submit_multisigned::SubmitMultisignedRequest;
use crate::types::{Signable, SigningContext, Transaction};

/// Builder for a [`SubmitRequest`] that signs the transaction inline.
///
/// # Example
/// ```rust,no_run
/// # use xrpl::types::{Transaction, SigningContext, builders::SubmitRequestBuilder};
/// #
/// # struct MyWallet;
/// #
/// # impl SigningContext for MyWallet {
/// #     type Error = anyhow::Error;
/// #
/// #     fn sign_transaction(&self, tx: &Transaction) -> anyhow::Result<String, Self::Error> {
/// #         // In a real implementation, this would sign the transaction and return the blob.
/// #         Ok("".to_string())
/// #     }
/// # }
/// #
/// # let tx: Transaction = todo!();
/// # let wallet = MyWallet;
///
/// let request = SubmitRequestBuilder::new(&tx, &wallet)
///     .fail_hard(true)
///     .build()
///     .expect("signing failed");
/// ```
pub struct SubmitRequestBuilder<'a, W: SigningContext> {
    tx: &'a Transaction,
    wallet: &'a W,
    fail_hard: Option<bool>,
}

impl<'a, W: SigningContext> SubmitRequestBuilder<'a, W> {
    /// Creates a new builder for the given transaction and signing wallet.
    pub fn new(tx: &'a Transaction, wallet: &'a W) -> Self {
        Self { tx, wallet, fail_hard: None }
    }

    /// Rejects the transaction instead of queuing it when it cannot enter the open ledger.
    pub fn fail_hard(mut self, value: bool) -> Self {
        self.fail_hard = Some(value);
        self
    }

    /// Signs the transaction and returns a ready-to-submit [`SubmitRequest`].
    pub fn build(self) -> Result<SubmitRequest, W::Error> {
        Ok(SubmitRequest {
            tx_blob: self.tx.sign_with(self.wallet)?,
            fail_hard: self.fail_hard,
        })
    }
}

/// Builder for a [`SubmitMultisignedRequest`].
///
/// Takes a fully assembled [`Transaction`] (with signatures attached via
/// [`Transaction::add_signature`]) and serializes it to JSON for submission.
///
/// # Example
/// ```rust,no_run
/// use xrpl::types::builders::SubmitMultisignedRequestBuilder;
/// # use xrpl::types::Transaction;
/// # let tx: Transaction = todo!();
/// let request = SubmitMultisignedRequestBuilder::new(&tx)
///     .fail_hard(true)
///     .build();
/// ```
///
/// [`Transaction::add_signature`]: crate::types::Transaction::add_signature
pub struct SubmitMultisignedRequestBuilder<'a> {
    tx: &'a Transaction,
    fail_hard: Option<bool>,
}

impl<'a> SubmitMultisignedRequestBuilder<'a> {
    /// Creates a new builder for the given assembled transaction.
    pub fn new(tx: &'a Transaction) -> Self {
        Self { tx, fail_hard: None }
    }

    /// Rejects the transaction instead of queuing it when it cannot enter the open ledger.
    pub fn fail_hard(mut self, value: bool) -> Self {
        self.fail_hard = Some(value);
        self
    }

    /// Serializes the transaction to JSON and returns a ready-to-submit [`SubmitMultisignedRequest`].
    pub fn build(self) -> SubmitMultisignedRequest {
        SubmitMultisignedRequest {
            tx_json: serde_json::to_value(self.tx)
                .expect("Transaction serialization failed"),
            fail_hard: self.fail_hard,
        }
    }
}
