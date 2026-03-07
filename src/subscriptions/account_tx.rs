use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::connection::SubscriptionClass;
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
    /// Create a new AccountTransactionsSubscription for the given accounts.
    pub fn new<I, S>(accounts: I) -> Result<Self, BuildError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let acc: Vec<String> = accounts.into_iter().map(|s| s.into()).collect();

        for addr in &acc {
            validate_address(addr)?;
        }

        Ok(Self { accounts_proposed: acc.clone(), accounts: acc })
    }
}

impl XrplRequest for AccountTransactionsSubscription {
    type Response = XrplResponse<AccountSubscriptionResponse>;
    const COMMAND: &str = "subscribe";
}

impl Hash for AccountTransactionsSubscription {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Sort accounts for deterministic hashing
        let mut sorted_accounts = self.accounts.clone();
        sorted_accounts.sort();

        // Hash sorted accounts for deterministic key generation
        "accounts".hash(state);
        for account in sorted_accounts {
            account.hash(state);
        }

        // Also hash proposed accounts
        let mut sorted_proposed = self.accounts_proposed.clone();
        sorted_proposed.sort();
        "accounts_proposed".hash(state);
        for account in sorted_proposed {
            account.hash(state);
        }
    }
}

impl XrplSubscription for AccountTransactionsSubscription {
    type Message = AccountTransactionMessage;

    fn matches(value: &Value) -> bool {
        value.get("type").and_then(|t| t.as_str()) == Some("transaction")
    }

    fn subscription_class(&self) -> SubscriptionClass {
        SubscriptionClass::Trading
    }
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
    pub meta: Option<TransactionMeta>,
    pub tx_json: Transaction,
    pub validated: bool,
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
        let sub = AccountTransactionsSubscription::new(accounts)
            .expect("Should accept valid addresses");

        assert_eq!(sub.accounts.len(), 2);
        assert_eq!(sub.accounts[0], "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    }

    #[test]
    fn test_account_subscription_new_invalid() {
        let accounts = vec!["not_an_address"];
        let result = AccountTransactionsSubscription::new(accounts);

        assert!(matches!(result, Err(BuildError::Validation(_))));
    }

    // test_account_unsubscription_new_valid and test_account_unsubscription_new_invalid removed: AccountTransactionsUnsubscription is obsolete. Unsubscription is now managed by SubscriptionHandle.

    #[test]
    fn test_subscription_key_sorting() {
        let addr1 = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
        let addr2 = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";

        let sub1 =
            AccountTransactionsSubscription::new(vec![addr2, addr1]).unwrap();
        let sub2 =
            AccountTransactionsSubscription::new(vec![addr1, addr2]).unwrap();

        // Hash keys should be deterministic regardless of input order
        assert_eq!(sub1.key(), sub2.key());
        // Hash key should be a valid u64
        assert!(sub1.key() > 0);
    }
}
