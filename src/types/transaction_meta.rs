use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use super::Amount;

/// Implemented by any type that carries [`TransactionMeta`].
///
/// Provides [`delivered_amount`](Self::delivered_amount) as a single, safe call
/// for reading the actual amount received by a payment — regardless of whether
/// the data comes from an `account_tx` response, a `tx` response, or a
/// subscription message.
///
/// # Examples
///
/// ```rust
/// use xrpl::types::HasTransactionMeta;
///
/// fn print_delivered(tx: &impl HasTransactionMeta) {
///     match tx.delivered_amount() {
///         Some(amount) => println!("Delivered: {amount}"),
///         None => println!("Not a payment transaction"),
///     }
/// }
/// ```
pub trait HasTransactionMeta {
    /// Returns the transaction metadata, if present.
    fn transaction_meta(&self) -> Option<&TransactionMeta>;

    /// Returns the actual amount delivered to the destination.
    ///
    /// Returns `None` for non-Payment transactions and for partial payments
    /// included in a validated ledger before 2014-01-20 (where the amount
    /// is not recoverable without inspecting `AffectedNodes`). Always use
    /// this instead of the transaction's `Amount` field to guard against
    /// partial-payment attacks.
    fn delivered_amount(&self) -> Option<&Amount> {
        self.transaction_meta()?.delivered_amount.as_ref()
    }
}

/// Execution metadata attached to every validated transaction.
///
/// Use [`delivered_amount`](Self::delivered_amount) instead of the transaction's
/// `Amount` field when crediting received payments — it reflects the actual amount
/// delivered and guards against partial-payment attacks. See [`HasTransactionMeta`]
/// for the possible states.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TransactionMeta {
    /// Ledger objects created, modified, or deleted by this transaction.
    pub affected_nodes: Vec<Value>,
    /// Position of this transaction within the ledger (zero-based).
    pub transaction_index: u32,
    /// Final transaction result code (e.g. `"tesSUCCESS"`).
    pub transaction_result: String,
    /// Actual amount delivered to the destination.
    ///
    /// Present only for Payment transactions. `None` for non-payment transactions
    /// and for pre-2014 partial payments where the amount cannot be recovered.
    /// Always use this instead of the transaction's `Amount` field.
    #[serde(
        rename = "delivered_amount",
        deserialize_with = "deserialize_delivered_amount",
        default
    )]
    pub delivered_amount: Option<Amount>,
}

fn deserialize_delivered_amount<'de, D>(
    d: D,
) -> Result<Option<Amount>, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Option::<Value>::deserialize(d)?;
    match v {
        None => Ok(None),
        Some(v) if v == "unavailable" => Ok(None),
        Some(v) => serde_json::from_value(v)
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}
