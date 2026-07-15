use crate::types::{
    validation::{validate_address, ValidationError},
    Amount, Memo, MemoWrapper, Signer, SignerWrapper, Transaction,
    TransactionType,
};

/// Default number of ledgers added to the current ledger index to compute
/// [`last_ledger_sequence`] in [`TransactionBuilder::fill`].
///
/// Each ledger closes in approximately 3-4 seconds, so 4 ledgers give a
/// submission window of roughly 12-16 seconds. Override this default per
/// transaction with [`TransactionBuilder::with_last_ledger_offset`] - tighten
/// it for latency-sensitive applications or widen it for high-congestion
/// environments.
///
/// [`last_ledger_sequence`]: https://xrpl.org/docs/references/protocol/transactions/common-fields#lastledgersequence
pub const LAST_LEDGER_OFFSET: u32 = 4;

/// Errors that can occur when building a transaction.
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    /// The fee amount was not provided in XRP drops.
    #[error("Fee must be in XRP drops")]
    FeeNotXRP,
    /// The transfer fee exceeded the allowed maximum of 50000 (50%).
    #[error("TransferFee must be between 0 and 50000")]
    InvalidTransferFee,
    /// `TicketSequence` is set but `Sequence` is not zero - the protocol requires
    /// `Sequence = 0` for all ticket-based transactions. Call `with_ticket_sequence`
    /// before `fill()` so the builder sets the correct sequence automatically, or
    /// set `Sequence = 0` explicitly via `with_sequence(0)`.
    #[error("Sequence must be 0 when TicketSequence is set")]
    TicketRequiresZeroSequence,
    /// `DIDSet` must have at least one of `DIDDocument`, `Data`, or `URI` set.
    #[error("At least one of DIDDocument, Data, or URI must be set")]
    DidSetEmpty,
    /// An address or amount validation check failed.
    #[error(transparent)]
    Validation(#[from] ValidationError),
}

/// Generic builder for any XRPL transaction type.
///
/// Holds the common transaction fields and delegates transaction-specific fields
/// to the `T` parameter. Use the concrete type aliases (e.g. [`PaymentBuilder`])
/// rather than constructing `TransactionBuilder` directly.
///
/// [`PaymentBuilder`]: crate::types::builders::PaymentBuilder
pub struct TransactionBuilder<T> {
    account: String,
    fee: Amount,
    sequence: u32,
    account_txn_id: Option<String>,
    flags: Option<u32>,
    last_ledger_sequence: Option<u32>,
    last_ledger_offset: Option<u32>,
    memos: Option<Vec<MemoWrapper>>,
    signers: Option<Vec<SignerWrapper>>,
    source_tag: Option<u32>,
    ticket_sequence: Option<u32>,
    pub(crate) transaction_type: T,
}

/// Implemented by each transaction-specific struct to validate fields and produce
/// the corresponding [`TransactionType`] variant.
pub trait TransactionTypeBuilder {
    /// The concrete `TransactionType` variant produced by this builder.
    type TransactionType;
    /// Validates transaction-specific fields before building.
    fn validate(&self) -> Result<(), BuildError>;
    /// Consumes `self` and returns the `TransactionType` variant.
    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError>;
}

impl<T: TransactionTypeBuilder<TransactionType = TransactionType>>
    TransactionBuilder<T>
{
    /// Creates a new builder with the mandatory common fields.
    pub fn init(
        account: impl AsRef<str>,
        sequence: u32,
        fee: impl Into<Amount>,
        transaction_type: T,
    ) -> Self {
        Self {
            account: account.as_ref().to_string(),
            account_txn_id: None,
            fee: fee.into(),
            flags: None,
            last_ledger_sequence: None,
            last_ledger_offset: None,
            memos: None,
            sequence,
            signers: None,
            source_tag: None,
            ticket_sequence: None,
            transaction_type,
        }
    }

    /// Sets the transaction flags bitmask.
    ///
    /// Each transaction type has a corresponding typed flags value that converts
    /// into a `u32` bitmask. Pass the typed constant or combine flags with `|`:
    ///
    /// | Builder | Flags type |
    /// |---------|-----------|
    /// | [`PaymentBuilder`](super::PaymentBuilder) | [`PaymentFlag`](super::super::PaymentFlag) / [`PaymentFlags`](super::super::PaymentFlags) |
    /// | [`TrustSetBuilder`](super::TrustSetBuilder) | [`TrustSetFlags`](super::super::TrustSetFlags) |
    /// | [`OfferCreateBuilder`](super::OfferCreateBuilder) | [`OfferCreateFlags`](super::super::OfferCreateFlags) |
    /// | [`AMMDepositBuilder`](super::AMMDepositBuilder) | [`AMMDepositFlags`](super::super::AMMDepositFlags) |
    /// | [`AMMWithdrawBuilder`](super::AMMWithdrawBuilder) | [`AMMWithdrawFlags`](super::super::AMMWithdrawFlags) |
    /// | [`NFTokenMintBuilder`](super::NFTokenMintBuilder) | [`NFTokenMintFlags`](super::super::NFTokenMintFlags) |
    /// | [`NFTokenCreateOfferBuilder`](super::NFTokenCreateOfferBuilder) | [`NFTokenCreateOfferFlags`](super::super::NFTokenCreateOfferFlags) |
    /// | [`MPTokenIssuanceCreateBuilder`](super::MPTokenIssuanceCreateBuilder) | [`MPTokenIssuanceCreateFlags`](super::super::MPTokenIssuanceCreateFlags) |
    /// | [`MPTokenAuthorizeBuilder`](super::MPTokenAuthorizeBuilder) | [`MPTokenAuthorizeFlags`](super::super::MPTokenAuthorizeFlags) |
    /// | [`MPTokenIssuanceSetBuilder`](super::MPTokenIssuanceSetBuilder) | [`MPTokenIssuanceSetAction`](super::super::MPTokenIssuanceSetAction) |
    /// | [`XChainModifyBridgeBuilder`](super::XChainModifyBridgeBuilder) | [`XChainModifyBridgeFlags`](super::super::XChainModifyBridgeFlags) |
    /// | [`PaymentChannelClaimBuilder`](super::PaymentChannelClaimBuilder) | [`PaymentChannelClaimAction`](super::super::PaymentChannelClaimAction) |
    ///
    /// A raw `u32` is also accepted for protocol flags not yet covered by a typed constant.
    ///
    /// # Example
    ///
    /// ```rust
    /// use xrpl::types::{PaymentFlag, TrustSetFlags, builders::{PaymentBuilder, TrustSetBuilder}};
    ///
    /// // Typed flags - combine with |
    /// let tx = PaymentBuilder::new("rSrc", "rDst", xrpl::xrp!(1))
    ///     .with_flags(PaymentFlag::PartialPayment | PaymentFlag::NoRippleDirect);
    ///
    /// // Struct-constant flags - TrustSet's LimitAmount must be an issued currency
    /// let tx = TrustSetBuilder::new(
    ///     "rSrc",
    ///     xrpl::issued!(0, "USD", "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"),
    /// )
    /// .with_flags(TrustSetFlags::SET_NO_RIPPLE | TrustSetFlags::SET_FREEZE);
    /// ```
    pub fn with_flags(mut self, flags: impl Into<u32>) -> Self {
        self.flags = Some(flags.into());
        self
    }

    /// Sets the last ledger sequence; the transaction is invalid after this ledger closes.
    pub fn with_last_ledger_sequence(mut self, sequence: u32) -> Self {
        self.last_ledger_sequence = Some(sequence);
        self
    }

    /// Overrides the ledger offset used by [`fill`] to compute `last_ledger_sequence`.
    ///
    /// [`fill`] sets `last_ledger_sequence = current_ledger + offset`, defaulting to
    /// [`LAST_LEDGER_OFFSET`]. Use this to widen the window for high-congestion
    /// environments or tighten it for latency-sensitive applications. Call before [`fill`].
    ///
    /// [`fill`]: Self::fill
    pub fn with_last_ledger_offset(mut self, offset: u32) -> Self {
        self.last_ledger_offset = Some(offset);
        self
    }

    /// Attaches human-readable memo entries to the transaction.
    ///
    /// Accepts any iterable of items convertible into [`Memo`], so users can
    /// pass `Vec<Memo>`, `[Memo; N]`, or any iterator yielding their own
    /// domain type that implements `Into<Memo>`.
    pub fn with_memos<I, M>(mut self, memos: I) -> Self
    where
        I: IntoIterator<Item = M>,
        M: Into<Memo>,
    {
        self.memos = Some(
            memos.into_iter().map(|m| MemoWrapper { memo: m.into() }).collect(),
        );
        self
    }

    /// Attaches multi-signature entries; required when the account uses a signer list.
    ///
    /// Accepts any iterable of items convertible into [`Signer`].
    pub fn with_signers<I, S>(mut self, signers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Signer>,
    {
        self.signers = Some(
            signers
                .into_iter()
                .map(|s| SignerWrapper { signer: s.into() })
                .collect(),
        );
        self
    }

    /// Sets an arbitrary source tag for identifying the originating application or user.
    pub fn with_source_tag(mut self, tag: u32) -> Self {
        self.source_tag = Some(tag);
        self
    }

    /// Uses a pre-allocated ticket sequence number instead of a regular sequence number.
    ///
    /// Call this **before** [`fill`] so the builder can skip the `account_info` round-trip
    /// and set `Sequence = 0` automatically. Calling it after `fill()` will cause
    /// [`build`] to return [`BuildError::TicketRequiresZeroSequence`].
    ///
    /// [`fill`]: Self::fill
    /// [`build`]: Self::build
    pub fn with_ticket_sequence(mut self, sequence: u32) -> Self {
        self.ticket_sequence = Some(sequence);
        self
    }

    /// Sets the hash of the previous transaction from this account for chaining guarantees.
    pub fn with_account_txn_id(mut self, id: impl AsRef<str>) -> Self {
        self.account_txn_id = Some(id.as_ref().to_string());
        self
    }

    /// Returns the current sequence number (as set by `init` or `fill`).
    pub fn sequence(&self) -> u32 {
        self.sequence
    }

    /// Overrides the sequence number.
    ///
    /// Use when managing sequence numbers manually, for example in multi-transaction
    /// workflows or when submitting transactions in parallel using pre-fetched values.
    pub fn with_sequence(mut self, sequence: u32) -> Self {
        self.sequence = sequence;
        self
    }

    /// Returns the current fee (as set by `init` or `fill`).
    ///
    /// Useful when you need to read back the filled fee to compute a derived fee,
    /// for example `(1 + N_signers) x base_fee` for multi-signature transactions.
    pub fn fee(&self) -> &Amount {
        &self.fee
    }

    /// Overrides the fee.
    ///
    /// Call after [`fill`] when the transaction requires a non-standard fee,
    /// such as multi-signature transactions where the fee must be `(1 + N) x base_fee`.
    ///
    /// [`fill`]: Self::fill
    pub fn with_fee(mut self, fee: impl Into<Amount>) -> Self {
        self.fee = fee.into();
        self
    }

    /// Fills in `fee` and `last_ledger_sequence` by querying the XRP Ledger, and
    /// also resolves `sequence` unless a ticket is already set.
    ///
    /// **Ticket mode** - call [`with_ticket_sequence`] before `fill()` to signal that
    /// the transaction uses a pre-allocated ticket instead of a regular sequence number.
    /// In that case `fill()` skips the `account_info` round-trip and sets `sequence = 0`
    /// automatically, as required by the protocol. No post-fill corrections needed.
    ///
    /// **Regular mode** - when no ticket sequence is set, all three fields (`sequence`,
    /// `fee`, `last_ledger_sequence`) are fetched concurrently from `account_info`,
    /// `fee`, and `ledger_current`.
    ///
    /// The fee is always set to `open_ledger_fee`. `last_ledger_sequence` is set to
    /// `ledger_current_index +` [`LAST_LEDGER_OFFSET`] (~12-16 s window). Override
    /// the offset per transaction with [`with_last_ledger_offset`].
    ///
    /// [`with_ticket_sequence`]: Self::with_ticket_sequence
    /// [`with_last_ledger_offset`]: Self::with_last_ledger_offset
    ///
    /// # Examples
    ///
    /// Regular transaction:
    /// ```rust,no_run
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// use xrpl::{Client, xrp, types::builders::PaymentBuilder};
    /// let client = Client::new("wss://xrplcluster.com");
    /// let tx = PaymentBuilder::new(
    ///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
    ///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
    ///     xrp!(1.0),
    /// )
    /// .fill(&client)
    /// .await?
    /// .build()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Ticket-based transaction:
    /// ```rust,no_run
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// use xrpl::{Client, xrp, types::builders::PaymentBuilder};
    /// let client = Client::new("wss://xrplcluster.com");
    /// let ticket_seq: u32 = 42; // obtained from account_objects after TicketCreate
    /// let tx = PaymentBuilder::new(
    ///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
    ///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
    ///     xrp!(1.0),
    /// )
    /// .with_ticket_sequence(ticket_seq)
    /// .fill(&client)
    /// .await?
    /// .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fill(
        self,
        client: &crate::Client,
    ) -> Result<Self, crate::XrplError> {
        use crate::request::{
            fee::FeeRequest, ledger_current::LedgerCurrentRequest,
        };

        let offset = self.last_ledger_offset.unwrap_or(LAST_LEDGER_OFFSET);

        match self.ticket_sequence {
            Some(_) => {
                let (fee_resp, ledger_resp) = tokio::try_join!(
                    client.request(&FeeRequest),
                    client.request(&LedgerCurrentRequest),
                )?;
                let fee = fee_resp.result()?.drops.open_ledger_fee;
                let last_ledger_sequence =
                    ledger_resp.result()?.ledger_current_index + offset;
                Ok(Self {
                    sequence: 0,
                    fee: Amount::Xrpl(fee),
                    last_ledger_sequence: Some(last_ledger_sequence),
                    ..self
                })
            }
            None => {
                use crate::util::next_sequence;
                let (seq, fee_resp, ledger_resp) = tokio::try_join!(
                    next_sequence(client, &self.account),
                    client.request(&FeeRequest),
                    client.request(&LedgerCurrentRequest),
                )?;
                let fee = fee_resp.result()?.drops.open_ledger_fee;
                let last_ledger_sequence =
                    ledger_resp.result()?.ledger_current_index + offset;
                Ok(Self {
                    sequence: seq,
                    fee: Amount::Xrpl(fee),
                    last_ledger_sequence: Some(last_ledger_sequence),
                    ..self
                })
            }
        }
    }

    /// Validates all fields and produces the final [`Transaction`].
    pub fn build(self) -> Result<Transaction, BuildError> {
        match &self.fee {
            Amount::IssuedCurrency { .. } => return Err(BuildError::FeeNotXRP),
            Amount::Mpt { .. } => return Err(BuildError::FeeNotXRP),
            Amount::Xrpl(_) => {}
        }

        if self.ticket_sequence.is_some() && self.sequence != 0 {
            return Err(BuildError::TicketRequiresZeroSequence);
        }

        validate_address(&self.account)?;

        self.transaction_type.validate()?;
        let transaction_type =
            self.transaction_type.build_transaction_type()?;

        Ok(Transaction {
            account: self.account,
            account_txn_id: self.account_txn_id,
            fee: self.fee.value().to_string(),
            flags: self.flags,
            last_ledger_sequence: self.last_ledger_sequence,
            memos: self.memos,
            sequence: self.sequence,
            signers: self.signers,
            source_tag: self.source_tag,
            ticket_sequence: self.ticket_sequence,
            signing_pub_key: None,
            txn_signature: None,
            hash: None,
            date: None,
            transaction_type,
        })
    }
}
