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
    use kamn_sdk::{service_signature_for_fields, AgentDid, ServiceApiClient, ServiceRequestAuth};

    support::ensure_live_test_env();

    let complete = r#"{"idempotency_key":"complete-1","completion_evidence_digest":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}"#;
    let release = r#"{"idempotency_key":"release-1"}"#;
    let requests = vec![
        support::ExpectedRequest {
            method: "POST",
            path: "/v1/tasks/task-1/complete".to_owned(),
            body: complete.to_owned(),
            sender_did: "kamn:did:agent:transition-contract".to_owned(),
            scope: "tasks:write",
            response_status: 200,
            response_body: r#"{"task_id":"task-1","state":"completed"}"#.to_owned(),
        },
        support::ExpectedRequest {
            method: "POST",
            path: "/v1/escrow/escrow-1/release".to_owned(),
            body: release.to_owned(),
            sender_did: "kamn:did:agent:transition-contract".to_owned(),
            scope: "escrow:write",
            response_status: 200,
            response_body: r#"{"escrow_id":"escrow-1","state":"released"}"#.to_owned(),
        },
    ];
    let (bind_addr, server) = support::spawn_expected_server(requests);
    let client = ServiceApiClient::connect(format!("http://{bind_addr}").as_str()).expect("client");
    let did = AgentDid::parse("kamn:did:agent:transition-contract").expect("did");
    let complete_auth = transition_auth(&did, 1, complete, "tasks:write");
    let release_auth = transition_auth(&did, 2, release, "escrow:write");

    client
        .complete_task_with_payload("task-1", complete, &complete_auth)
        .expect("complete response");
    client
        .release_escrow_with_payload("escrow-1", release, &release_auth)
        .expect("release response");
    server.join().expect("server").expect("request contract");

    fn transition_auth(did: &AgentDid, nonce: u64, body: &str, scope: &str) -> ServiceRequestAuth {
        let signature = service_signature_for_fields(did, nonce, "kamn-sdk-live", "1", body)
            .expect("signature");
        ServiceRequestAuth::new_with_scope(did.clone(), nonce, signature, Some(scope))
            .expect("auth")
    }
}
