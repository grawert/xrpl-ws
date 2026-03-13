mod common;

use xrpl::{Client, request::server_state::ServerStateRequest};
use common::*;

#[ignore]
#[tokio::test]
async fn test_server_state_request() {
    let client = Client::new(&server_url());
    let response = client.request(ServerStateRequest).await.unwrap();
    let result = response.result().unwrap();
    let state = &result.state;

    // Assert that some key fields are present and valid
    assert!(
        !state.build_version.is_empty(),
        "build_version should not be empty"
    );
    assert!(
        !state.complete_ledgers.is_empty(),
        "complete_ledgers should not be empty"
    );
    assert!(state.peers > 0, "peers should be greater than 0");
    assert!(!state.server_state.is_empty(), "server_state should not be empty");
    assert!(state.uptime > 0, "uptime should be greater than 0");
    assert!(
        state.validation_quorum > 0,
        "validation_quorum should be greater than 0"
    );
    assert!(
        state.validated_ledger.seq > 0,
        "validated_ledger.seq should be greater than 0"
    );
    // Option fields: print a warning if missing, but do not fail the test
    if state.reserve_inc_xrp.is_none() {
        eprintln!("Warning: reserve_inc_xrp is None");
    }
    if state.reserve_base_xrp.is_none() {
        eprintln!("Warning: reserve_base_xrp is None");
    }
}
