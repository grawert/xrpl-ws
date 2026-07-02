mod common;

use xrpl::Client;
use xrpl::request::account_tx::AccountTxRequest;
use xrpl::types::{HasTransactionMeta, Transaction, TransactionType};
use common::*;

const DEFAULT_TX_LIMIT: u32 = 10;

#[tokio::test]
async fn test_account_tx() {
    let client = Client::new(server_url());
    let request =
        AccountTxRequest::new(sender_address()).with_limit(DEFAULT_TX_LIMIT);

    let response =
        client.request(&request).await.expect("Failed to request account_tx");
    let result = response.result().expect("Expected transactions in response");

    assert!(!result.transactions.is_empty());

    for tx in &result.transactions {
        let typed: Transaction =
            serde_json::from_value(tx.tx_json.clone().unwrap())
                .expect("Failed to deserialize transaction");

        match &typed.transaction_type {
            TransactionType::Payment(p) => match tx.delivered_amount() {
                Some(amount) => println!(
                    "[Payment] {} -> {} delivered: {amount}",
                    typed.account, p.destination
                ),
                None => println!(
                    "[Payment] {} -> {} delivered: n/a",
                    typed.account, p.destination
                ),
            },
            _ => {
                println!(
                    "[{}] account: {}",
                    typed.transaction_type_name(),
                    typed.account
                );
            }
        }
    }
}
