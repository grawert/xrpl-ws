mod common;

use xrpl::*;
use xrpl::request::account_objects::{AccountObjectsRequest, AccountObjectType};
use common::*;
use serde_json::json;

macro_rules! test_account_object_type {
    ($test_name:ident, $variant:expr) => {
        #[ignore]
        #[tokio::test]
        async fn $test_name() {
            let client = XrplClient::new(SERVER_URL).await.unwrap();
            let request = AccountObjectsRequest {
                account: TEST_ACCOUNT.to_string(),
                ledger_index: Some(json!("validated")),
                limit: Some(10),
                kind: Some($variant),
                ..Default::default()
            };

            let response =
                client.request(request).await.expect(stringify!($variant));
            response.result().expect(&format!(
                "Expected {} in response",
                stringify!($variant)
            ));
        }
    };
}

test_account_object_type!(bridge, AccountObjectType::Bridge);
test_account_object_type!(check, AccountObjectType::Check);
test_account_object_type!(deposit, AccountObjectType::DepositPreauth);
test_account_object_type!(escrow, AccountObjectType::Escrow);
test_account_object_type!(mptoken, AccountObjectType::MPToken);
test_account_object_type!(nft_offer, AccountObjectType::NFTokenOffer);
test_account_object_type!(nft_page, AccountObjectType::NFTokenPage);
test_account_object_type!(offer, AccountObjectType::Offer);
test_account_object_type!(paychannel, AccountObjectType::PayChannel);
test_account_object_type!(state, AccountObjectType::RippleState);
test_account_object_type!(signer_list, AccountObjectType::SignerList);
test_account_object_type!(ticket, AccountObjectType::Ticket);
