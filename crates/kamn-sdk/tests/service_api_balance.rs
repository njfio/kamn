#[path = "support/live_transport_task_escrow.rs"]
mod support;

use kamn_sdk::{
    service_signature_for_fields, AgentDid, ServiceAgentBalance, ServiceApiClient,
    ServiceRequestAuth,
};
use std::thread;

use support::{
    ensure_live_test_env, expected_request, reserve_loopback_addr, run_contract_server,
    wait_for_server_ready, ExpectedRequest,
};

const REQUESTER_DID: &str = "kamn:did:agent:live-requester";
const TARGET_DID: &str = "kamn:did:agent:balance-target";
const REQUEST_NONCE: u64 = 7;
const AGENTS_READ_SCOPE: &str = "agents:read";

fn auth() -> ServiceRequestAuth {
    let sender = AgentDid::parse(REQUESTER_DID).expect("requester did should parse");
    let signature = service_signature_for_fields(&sender, REQUEST_NONCE, "kamn-sdk-live", "1", "")
        .expect("service signature should build");
    ServiceRequestAuth::new_with_scope(sender, REQUEST_NONCE, signature, Some(AGENTS_READ_SCOPE))
        .expect("request auth should build")
}

#[test]
fn spec_c01_service_api_client_reads_agent_balance_over_agents_read_route() {
    ensure_live_test_env();
    let bind_addr = reserve_loopback_addr();
    let expected_requests = vec![ExpectedRequest {
        sender_did: REQUESTER_DID.to_owned(),
        scope: AGENTS_READ_SCOPE,
        response_body: format!(r#"{{"did":"{TARGET_DID}","balance":100}}"#),
        ..expected_request("GET", format!("/v1/agents/{TARGET_DID}/balance").as_str(), "")
    }];
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_contract_server(server_addr, expected_requests));
    wait_for_server_ready();

    let endpoint = format!("http://{bind_addr}");
    let client = ServiceApiClient::connect(endpoint.as_str()).expect("service client should connect");
    let balance: ServiceAgentBalance = client
        .get_agent_balance(TARGET_DID, &auth())
        .expect("agent balance should resolve");

    assert_eq!(balance.did, TARGET_DID);
    assert_eq!(balance.balance, 100);
    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "service client balance route server should satisfy request budget"
    );
}
