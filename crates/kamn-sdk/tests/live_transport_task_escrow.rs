#[path = "live_transport_task_escrow/fail_closed_contract_tests.rs"]
mod fail_closed_contract_tests;
#[path = "live_transport_task_escrow/happy_path_contract_tests.rs"]
mod happy_path_contract_tests;
#[path = "live_transport_task_escrow/malformed_payload_contract_tests.rs"]
mod malformed_payload_contract_tests;
#[path = "live_transport_task_escrow/support.rs"]
mod support;

#[test]
fn projection_routes_preserve_server_generated_json() {
    use kamn_sdk::{service_signature_for_fields, AgentDid, ServiceApiClient, ServiceRequestAuth};

    support::ensure_live_test_env();
    let participant = r#"{"view_scope":"participant-private","public_commitment":"fnv1a64:a","task_receipt_ids":["receipt-1"]}"#;
    let verifier = r#"{"view_scope":"restricted-public","public_commitment":"fnv1a64:a"}"#;
    let requests = vec![
        projection_request("participant-view", participant),
        projection_request("verifier-view", verifier),
    ];
    let (bind_addr, server) = support::spawn_expected_server(requests);
    let client = ServiceApiClient::connect(format!("http://{bind_addr}").as_str()).expect("client");
    let did = AgentDid::parse("kamn:did:agent:projection-contract").expect("did");
    let participant_auth = projection_auth(&did, 1);
    let verifier_auth = projection_auth(&did, 2);

    assert_eq!(
        client
            .get_task_participant_projection("task-1", &participant_auth)
            .expect("participant projection"),
        participant
    );
    assert_eq!(
        client
            .get_task_verifier_projection("task-1", &verifier_auth)
            .expect("verifier projection"),
        verifier
    );
    server.join().expect("server").expect("request contract");

    fn projection_request(view: &str, response_body: &str) -> support::ExpectedRequest {
        support::ExpectedRequest {
            method: "GET",
            path: format!("/v1/tasks/task-1/{view}"),
            body: String::new(),
            sender_did: "kamn:did:agent:projection-contract".to_owned(),
            scope: "tasks:read",
            response_status: 200,
            response_body: response_body.to_owned(),
        }
    }

    fn projection_auth(did: &AgentDid, nonce: u64) -> ServiceRequestAuth {
        let signature =
            service_signature_for_fields(did, nonce, "kamn-sdk-live", "1", "").expect("signature");
        ServiceRequestAuth::new_with_scope(did.clone(), nonce, signature, Some("tasks:read"))
            .expect("auth")
    }
}

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
