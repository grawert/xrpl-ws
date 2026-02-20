use crate::types::{
    Amount, Memo, MemoWrapper, Signer, SignerWrapper, Transaction,
    TransactionType, validate_address,
};

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("Fee must be in XRP drops")]
    FeeNotXRP,
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    #[error("Invalid field: {0}")]
    InvalidField(String),
}

pub struct TransactionBuilder<T> {
    account: String,
    fee: Amount,
    sequence: u32,
    account_txn_id: Option<String>,
    flags: Option<u32>,
    last_ledger_sequence: Option<u32>,
    memos: Option<Vec<MemoWrapper>>,
    signers: Option<Vec<SignerWrapper>>,
    source_tag: Option<u32>,
    ticket_sequence: Option<u32>,
    pub(crate) transaction_type: T,
}

pub trait TransactionTypeBuilder {
    type TransactionType;
    fn validate(&self) -> Result<(), BuildError>;
    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError>;
}

impl<T: TransactionTypeBuilder<TransactionType = TransactionType>>
    TransactionBuilder<T>
{
    pub fn init(
        account: String,
        sequence: u32,
        fee: Amount,
        transaction_type: T,
    ) -> Self {
        Self {
            account,
            account_txn_id: None,
            fee,
            flags: None,
            last_ledger_sequence: None,
            memos: None,
            sequence,
            signers: None,
            source_tag: None,
            ticket_sequence: None,
            transaction_type,
        }
    }

    pub fn with_flags(mut self, flags: u32) -> Self {
        self.flags = Some(flags);
        self
    }

    pub fn with_last_ledger_sequence(mut self, sequence: u32) -> Self {
        self.last_ledger_sequence = Some(sequence);
        self
    }

    pub fn with_memos(mut self, memos: Vec<Memo>) -> Self {
        self.memos =
            Some(memos.into_iter().map(|memo| MemoWrapper { memo }).collect());
        self
    }

    pub fn with_signers(mut self, signers: Vec<Signer>) -> Self {
        self.signers = Some(
            signers
                .into_iter()
                .map(|signer| SignerWrapper { signer })
                .collect(),
        );
        self
    }

    pub fn with_source_tag(mut self, tag: u32) -> Self {
        self.source_tag = Some(tag);
        self
    }

    pub fn with_ticket_sequence(mut self, sequence: u32) -> Self {
        self.ticket_sequence = Some(sequence);
        self
    }

    pub fn with_account_txn_id(mut self, id: String) -> Self {
        self.account_txn_id = Some(id);
        self
    }

    pub fn build(self) -> Result<Transaction, BuildError> {
        match &self.fee {
            Amount::IssuedCurrency { .. } => return Err(BuildError::FeeNotXRP),
            Amount::Xrpl(_) => {}
        }
        validate_address(&self.account, "account")?;
        self.transaction_type.validate()?;
        let transaction_type =
            self.transaction_type.build_transaction_type()?;

        Ok(Transaction {
            account: self.account,
            account_txn_id: self.account_txn_id,
            fee: self.fee.to_string(),
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
            transaction_type,
        })
    }
}
