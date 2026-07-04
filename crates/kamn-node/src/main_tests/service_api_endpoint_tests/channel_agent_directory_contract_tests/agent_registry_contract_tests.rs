use super::super::*;
use super::support::{
    build_directory_snapshot, query_agent_profile, raw_signed_request, register_agent,
    search_agents, unique_named_state_file, RawSignedRequest,
};

#[test]
fn integration_service_api_endpoint_registers_agent_metadata_idempotently_and_conflicts_on_mismatch(
) {
    let _env = acquire_service_api_test_env();
    let snapshot = build_directory_snapshot("127.0.0.1:34121");
    let bind_addr = reserve_loopback_addr();
    let caller_did = "kamn:did:agent:register-agent-profile";
    let registered_caller_did = test_service_api_sender_did(caller_did);
    let registration_body =
        r#"{"agent_type":"assistant","model_family":"gpt-5","capabilities":["text","code"]}"#;

    let registration_payload = register_agent(
        &snapshot,
        bind_addr.as_str(),
        caller_did,
        201,
        registration_body,
    );
    let duplicate_response = raw_signed_request(
        &snapshot,
        bind_addr.as_str(),
        RawSignedRequest {
            max_requests: 1,
            method: "POST",
            path: "/v1/agents/register",
            sender_did: caller_did,
            nonce: 202,
            body: registration_body,
            extra_headers: &[],
        },
    );
    let mismatch_response = raw_signed_request(
        &snapshot,
        bind_addr.as_str(),
        RawSignedRequest {
            max_requests: 1,
            method: "POST",
            path: "/v1/agents/register",
            sender_did: caller_did,
            nonce: 203,
            body: r#"{"agent_type":"assistant","model_family":"gpt-5o","capabilities":["text"]}"#,
            extra_headers: &[],
        },
    );
    let query_payload = query_agent_profile(
        &snapshot,
        bind_addr.as_str(),
        "kamn:did:agent:register-agent-profile-reader",
        301,
        registered_caller_did.as_str(),
    );

    assert_eq!(registration_payload.did, registered_caller_did);
    assert_eq!(registration_payload.agent_type, "assistant");
    assert_eq!(registration_payload.model_family, "gpt-5");
    assert_eq!(
        registration_payload.capabilities,
        vec!["text".to_owned(), "code".to_owned()]
    );
    assert!(duplicate_response.contains("HTTP/1.1 201 Created"));
    assert!(mismatch_response.contains("HTTP/1.1 409 Conflict"));
    assert_eq!(
        parse_error_envelope(extract_http_response_body(mismatch_response.as_str())).reason_code,
        "service_api_agent_registration_conflict"
    );
    assert_eq!(query_payload.did, registered_caller_did);
    assert_eq!(query_payload.agent_type, "assistant");
    assert_eq!(query_payload.model_family, "gpt-5");
    assert_eq!(
        query_payload.capabilities,
        vec!["text".to_owned(), "code".to_owned()]
    );
}

#[test]
fn integration_service_api_endpoint_searches_registered_agent_metadata() {
    let _env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-agent-search");
    let state_file_text = state_file.to_string_lossy().to_string();
    let _state_file_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_STATE_FILE",
        Some(state_file_text.as_str()),
    );
    let snapshot = build_directory_snapshot("127.0.0.1:34121");
    let bind_addr = reserve_loopback_addr();

    for (nonce, caller_did, registration_body) in [
        (
            401_u64,
            "kamn:did:agent:search-alpha",
            r#"{"agent_type":"assistant","model_family":"gpt-5","capabilities":["text","code"]}"#,
        ),
        (
            402_u64,
            "kamn:did:agent:search-beta",
            r#"{"agent_type":"assistant","model_family":"gpt-4.1","capabilities":["text"]}"#,
        ),
    ] {
        let response = raw_signed_request(
            &snapshot,
            bind_addr.as_str(),
            RawSignedRequest {
                max_requests: 1,
                method: "POST",
                path: "/v1/agents/register",
                sender_did: caller_did,
                nonce,
                body: registration_body,
                extra_headers: &[],
            },
        );
        assert!(response.contains("HTTP/1.1 201 Created"));
    }

    let search_payload = search_agents(
        &snapshot,
        bind_addr.as_str(),
        "kamn:did:agent:search-reader",
        403,
        r#"{"capability":"code","model_family":"gpt-5"}"#,
    );
    assert_eq!(search_payload.len(), 1);
    assert_eq!(
        search_payload[0].did,
        test_service_api_sender_did("kamn:did:agent:search-alpha")
    );
    assert_eq!(search_payload[0].agent_type, "assistant");
    assert_eq!(search_payload[0].model_family, "gpt-5");
    assert_eq!(
        search_payload[0].capabilities,
        vec!["text".to_owned(), "code".to_owned()]
    );
    let _ = fs::remove_file(state_file);
}

#[test]
fn integration_service_api_endpoint_rejects_invalid_agent_search_payload() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_directory_snapshot("127.0.0.1:34121");
    let response = raw_signed_request(
        &snapshot,
        reserve_loopback_addr().as_str(),
        RawSignedRequest {
            max_requests: 1,
            method: "POST",
            path: "/v1/agents/search",
            sender_did: "kamn:did:agent:search-invalid",
            nonce: 404,
            body: r#"{"capability":"   "}"#,
            extra_headers: &[("X-KAMN-Authz-Scope", "agents:read")],
        },
    );

    assert!(response.contains("HTTP/1.1 400 Bad Request"));
    let payload: ServiceApiErrorBody =
        parse_service_api_payload(extract_http_response_body(response.as_str()))
            .expect("error payload should deserialize");
    assert_eq!(payload.error, "bad-request");
    assert_eq!(
        payload.reason_code,
        "service_api_agent_search_payload_invalid"
    );
    assert!(payload
        .message
        .contains("agent search payload capability must not be empty when provided"));
}
