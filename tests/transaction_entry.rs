mod common;

use xrpl::Client;
use xrpl::request::account_tx::AccountTxRequest;
use xrpl::request::transaction_entry::TransactionEntryRequest;
use xrpl::request::XrplRequest;
use common::*;

/// Fetches the most recent validated transaction for the test account.
/// Returns `(tx_hash, ledger_index)` if found.
async fn most_recent_tx(client: &xrpl::Client) -> Option<(String, u32)> {
    let request = AccountTxRequest::new(sender_address())
        .with_limit(1)
        .with_ledger_index_min(-1)
        .with_ledger_index_max(-1);
    let result = client.request(&request).await.ok()?.result().ok()?;

    let tx = result.transactions.into_iter().next()?;
    let hash = tx.tx_json.as_ref()?.get("hash")?.as_str()?.to_string();
    let ledger_index =
        tx.tx_json.as_ref()?.get("ledger_index")?.as_u64()? as u32;
    Some((hash, ledger_index))
}

#[tokio::test]
async fn test_transaction_entry() {
    let client = Client::new(server_url());

    let Some((hash, ledger_index)) = most_recent_tx(&client).await else {
        // No transactions on this account yet
        return;
    };

    let request = TransactionEntryRequest::new(hash.clone())
        .with_ledger_index(ledger_index);
    let result = client.request(&request).await.unwrap().result().unwrap();

    assert_eq!(result.tx_json.hash.as_deref(), Some(hash.as_str()));
}

#[test]
fn test_transaction_entry_serializes() {
    const TX_HASH: &str =
        "E3FE6EA3D48F0C2B639448020EA4F03D4F4F8FFDB243A852A0F59177921B4879";
    const LEDGER_INDEX: u32 = 12345;

    let req =
        TransactionEntryRequest::new(TX_HASH).with_ledger_index(LEDGER_INDEX);
    let json = req.to_value().expect("Failed to serialize request");
    assert_eq!(json["command"], "transaction_entry");
    assert_eq!(json["tx_hash"], TX_HASH);
    assert_eq!(json["ledger_index"], LEDGER_INDEX);
    assert!(json["ledger_hash"].is_null());
}
