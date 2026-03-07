#[path = "support/live_transport_task_escrow.rs"]
mod support;

use kamn_sdk::{KamnAgent, LiveTransportKamnClient, TokenAmount};
use std::thread;

use support::{
    did, ensure_live_test_env, expected_request, reserve_loopback_addr, run_contract_server,
    wait_for_server_ready, ExpectedRequest,
};

const REQUESTER_DID: &str = "kamn:did:agent:live-requester";
const TARGET_DID: &str = "kamn:did:agent:balance-target";

#[test]
fn spec_c01_live_transport_balance_route_executes_network_contract() {
    ensure_live_test_env();
    let bind_addr = reserve_loopback_addr();
    let expected_requests = vec![ExpectedRequest {
        sender_did: REQUESTER_DID.to_owned(),
        scope: "agents:read",
        response_status: 200,
        response_body: format!(r#"{{"did":"{TARGET_DID}","balance":100}}"#),
        ..expected_request("GET", format!("/v1/agents/{TARGET_DID}/balance").as_str(), "")
    }];
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_contract_server(server_addr, expected_requests));
    wait_for_server_ready();

    let endpoint = format!("http://{bind_addr}");
    let client =
        LiveTransportKamnClient::connect(endpoint.as_str()).expect("live client should connect");
    let balance = client
        .balance(&did("balance-target"))
        .expect("live balance should succeed");
    assert_eq!(balance, TokenAmount(100));

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "live transport balance server should satisfy request budget"
    );
}
