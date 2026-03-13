use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::request::{XrplRequest, XrplResponse, XrplSubscription};
use crate::types::{
    Transaction, validation::validate_address, builders::BuildError,
};

/// Subscription request for account transaction events.
#[derive(Debug, Serialize, Clone)]
pub struct AccountTransactionsSubscription {
    pub accounts: Vec<String>,
    pub accounts_proposed: Vec<String>,
}

impl AccountTransactionsSubscription {
    /// Subscribe to validated and unvalidated transactions for the given accounts.
    pub fn proposed<I, S>(accounts: I) -> Result<Self, BuildError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let acc: Vec<String> = accounts.into_iter().map(|s| s.into()).collect();

        for addr in &acc {
            validate_address(addr)?;
        }

        Ok(Self { accounts: vec![], accounts_proposed: acc })
    }

    /// Subscribe to validated only transactions for the given accounts.
    pub fn validated<I, S>(accounts: I) -> Result<Self, BuildError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let acc: Vec<String> = accounts.into_iter().map(|s| s.into()).collect();

        for addr in &acc {
            validate_address(addr)?;
        }

        Ok(Self { accounts: acc, accounts_proposed: vec![] })
    }
}

impl XrplRequest for AccountTransactionsSubscription {
    type Response = XrplResponse<AccountSubscriptionResponse>;
    const COMMAND: &str = "subscribe";
}

impl XrplSubscription for AccountTransactionsSubscription {
    type Message = AccountTransactionMessage;
}

#[derive(Debug, Deserialize)]
pub struct AccountSubscriptionResponse {
    pub accounts: Option<Vec<String>>,
    pub accounts_proposed: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountTransactionMessage {
    pub close_time_iso: Option<String>,
    #[serde(rename = "type")]
    pub kind: String,
    pub engine_result: String,
    pub engine_result_code: i32,
    pub engine_result_message: String,
    pub hash: String,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<u32>,
    pub ledger_current_index: Option<u32>,
    pub meta: Option<TransactionMeta>,
    pub tx_json: Transaction,
    pub validated: bool,
    pub ctid: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TransactionMeta {
    pub affected_nodes: Vec<Value>,
    pub transaction_index: u32,
    pub transaction_result: String,
    #[serde(rename = "delivered_amount")]
    pub delivered_amount: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_subscription_new_valid() {
        let accounts = vec![
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        ];
        let sub = AccountTransactionsSubscription::proposed(accounts)
            .expect("Should accept valid addresses");

        assert_eq!(sub.accounts_proposed.len(), 2);
        assert_eq!(
            sub.accounts_proposed[0],
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
        );

        // accounts should be empty for proposed subscriptions
        assert_eq!(sub.accounts.len(), 0);
    }

    #[test]
    fn test_account_subscription_validated() {
        let accounts = vec![
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        ];
        let sub = AccountTransactionsSubscription::validated(accounts)
            .expect("Should accept valid addresses");

        assert_eq!(sub.accounts.len(), 2);
        assert_eq!(sub.accounts[0], "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");

        // accounts_proposed should be empty for validated subscriptions
        assert_eq!(sub.accounts_proposed.len(), 0);
    }

    #[test]
    fn test_account_subscription_new_invalid() {
        let accounts = vec!["not_an_address"];
        let result = AccountTransactionsSubscription::proposed(accounts);

        assert!(matches!(result, Err(BuildError::Validation(_))));
    }
}
