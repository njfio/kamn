#[path = "live_transport_task_escrow/fail_closed_contract_tests.rs"]
mod fail_closed_contract_tests;
#[path = "live_transport_task_escrow/happy_path_contract_tests.rs"]
mod happy_path_contract_tests;
#[path = "live_transport_task_escrow/malformed_payload_contract_tests.rs"]
mod malformed_payload_contract_tests;
#[path = "live_transport_task_escrow/support.rs"]
mod support;

#[test]
fn transition_routes_preserve_canonical_payloads() {
    use kamn_sdk::{AgentDid, ServiceApiClient, ServiceRequestAuth};

    let complete = r#"{"idempotency_key":"complete-1","completion_evidence_digest":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}"#;
    let release = r#"{"idempotency_key":"release-1"}"#;
    let requests = vec![
        support::expected_request("POST", "/v1/tasks/task-1/complete", complete),
        support::expected_request("POST", "/v1/escrow/escrow-1/release", release),
    ];
    let (bind_addr, server) = support::spawn_expected_server(requests);
    let client = ServiceApiClient::connect(format!("http://{bind_addr}").as_str()).expect("client");
    let auth = ServiceRequestAuth::new(
        AgentDid::parse("kamn:did:agent:transition-contract").expect("did"),
        1,
        "signature".to_owned(),
    )
    .expect("auth");

    client
        .complete_task_with_payload("task-1", complete, &auth)
        .expect("complete response");
    client
        .release_escrow_with_payload("escrow-1", release, &auth)
        .expect("release response");
    server.join().expect("server").expect("request contract");
}
