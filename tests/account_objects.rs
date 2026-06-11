mod common;

use xrpl::*;
use xrpl::request::account_objects::{AccountObjectsRequest, AccountObjectType};
use common::*;

macro_rules! test_account_object_type {
    ($test_name:ident, $variant:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let client = Client::new(&server_url());
            let request = AccountObjectsRequest::new(sender_address())
                .with_ledger_index("validated")
                .with_limit(10)
                .with_kind($variant);

            let response =
                client.request(&request).await.expect(stringify!($variant));
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
