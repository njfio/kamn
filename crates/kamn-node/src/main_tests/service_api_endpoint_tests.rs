use super::*;
use crate::service_api_endpoint::{
    parse_service_api_payload, upsert_service_api_relayed_message_from_daemon,
    ServiceApiAgentGetBody, ServiceApiChannelCreateBody, ServiceApiErrorBody, ServiceApiHealthBody,
    ServiceApiMessageCreateBody, ServiceApiRelaySpoolEntry, ServiceApiTaskCreateBody,
    DEFAULT_SERVICE_API_BODY_LIMIT_BYTES, DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
    DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND, SERVICE_API_AUTH_REASON_CODES_CSV,
    SERVICE_API_AUTH_REASON_TAXONOMY_VERSION, SERVICE_API_LIFECYCLE_REJECTION_REASON_CODES_CSV,
    SERVICE_API_ROUTE_AUTHZ_MATRIX_PROTECTED_ROUTE_COUNT,
    SERVICE_API_ROUTE_AUTHZ_MATRIX_PUBLIC_ROUTE_COUNT,
    SERVICE_API_ROUTE_AUTHZ_MATRIX_SCHEMA_VERSION,
    SERVICE_API_ROUTE_AUTHZ_MATRIX_TOTAL_ROUTE_COUNT,
    SERVICE_API_SCOPE_POLICY_FIXTURE_SCHEMA_VERSION, SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV,
    SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION, SERVICE_API_WEBSOCKET_REASON_CODES_CSV,
};
use kamn_core::AgentDid;
use kamn_core::{
    cross_store_replay_reason_codes_csv, cross_store_replay_reason_taxonomy_version,
    service_auth_public_key_hex_from_private_key_hex, service_auth_sign_with_private_key_hex,
};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, MutexGuard};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[path = "service_api_endpoint_tests/auth_scope_contract_tests.rs"]
mod auth_scope_contract_tests;
#[path = "service_api_endpoint_tests/balance_contract_tests.rs"]
mod balance_contract_tests;
#[path = "service_api_endpoint_tests/bridge_persistence_restart_contract_tests.rs"]
mod bridge_persistence_restart_contract_tests;
#[path = "service_api_endpoint_tests/channel_agent_directory_contract_tests.rs"]
mod channel_agent_directory_contract_tests;
#[path = "service_api_endpoint_tests/content_lifecycle_restart_contract_tests.rs"]
mod content_lifecycle_restart_contract_tests;
#[path = "service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests.rs"]
mod ingress_guard_lifecycle_contract_tests;
#[path = "service_api_endpoint_tests/mailbox_relay_delivery_contract_tests.rs"]
mod mailbox_relay_delivery_contract_tests;
#[path = "service_api_endpoint_tests/message_persistence_contract_tests.rs"]
mod message_persistence_contract_tests;
#[path = "service_api_endpoint_tests/route_render_contract_tests.rs"]
mod route_render_contract_tests;
#[path = "service_api_endpoint_tests/task_escrow_persistence_contract_tests.rs"]
mod task_escrow_persistence_contract_tests;
#[path = "service_api_endpoint_tests/transport_surface_observability_contract_tests.rs"]
mod transport_surface_observability_contract_tests;
#[path = "service_api_endpoint_tests/websocket_contract_tests.rs"]
mod websocket_contract_tests;

#[path = "service_api_endpoint_tests/shared_support.rs"]
mod shared_support;

pub(crate) use shared_support::*;

#[test]
fn regression_service_api_env_lock_recovers_from_signer_lock_poison() {
    // Regression: #5199
    let _ = std::panic::catch_unwind(|| {
        let _lock = lock_signer_env_guard();
        panic!("intentional signer env lock poison");
    });
    let _env = acquire_service_api_test_env();
}

#[test]
fn unit_service_api_endpoint_serde_payload_roundtrip_contracts() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34060".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let health = render_service_api_endpoint_response(&snapshot, "GET", "/healthz", "");
    let health_payload: ServiceApiHealthBody =
        parse_service_api_payload(health.body.as_str()).expect("health payload should deserialize");
    assert_eq!(health_payload.status, "ok");
    assert_eq!(health_payload.runtime_mode, "api");

    let send = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/messages/send",
        "{\"message\":\"serde\"}",
    );
    let send_payload: ServiceApiMessageCreateBody =
        parse_service_api_payload(send.body.as_str()).expect("send payload should deserialize");
    assert_eq!(send_payload.status, "created");
    assert!(send_payload.message_id.starts_with("msg-local-"));

    let channel = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/channels/create",
        "{\"name\":\"alpha\"}",
    );
    let channel_payload: ServiceApiChannelCreateBody =
        parse_service_api_payload(channel.body.as_str())
            .expect("channel payload should deserialize");
    assert_eq!(channel_payload.status, "created");

    let task = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/tasks/create",
        "{\"task\":\"x\"}",
    );
    let task_payload: ServiceApiTaskCreateBody =
        parse_service_api_payload(task.body.as_str()).expect("task payload should deserialize");
    assert_eq!(task_payload.state, "submitted");

    let agent = render_service_api_endpoint_response(
        &snapshot,
        "GET",
        "/v1/agents/kamn:did:agent:alpha",
        "",
    );
    let agent_payload: ServiceApiAgentGetBody =
        parse_service_api_payload(agent.body.as_str()).expect("agent payload should deserialize");
    assert_eq!(agent_payload.did, "kamn:did:agent:alpha");
    assert_eq!(agent_payload.reputation_score, 500);
    let agent_json: Value =
        serde_json::from_str(agent.body.as_str()).expect("agent payload should parse as json");
    assert_eq!(agent_json["agent_type"], "service-agent");
    assert_eq!(agent_json["model_family"], "service-api");
    assert_eq!(
        agent_json["capabilities"],
        serde_json::json!(["profile:read"])
    );
}

#[test]
fn unit_service_api_endpoint_error_envelopes_use_reason_code_and_message_contracts() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34061".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let websocket_required =
        render_service_api_endpoint_response(&snapshot, "GET", "/v1/events/ws", "");
    assert_eq!(websocket_required.status_code, 400);
    let websocket_required_payload = parse_error_envelope(websocket_required.body.as_str());
    assert_eq!(websocket_required_payload.error, "bad-request");
    assert_eq!(
        websocket_required_payload.reason_code,
        route_render_contract_tests::websocket_upgrade_required_reason_code()
    );
    assert!(websocket_required_payload
        .message
        .contains("websocket upgrade required"));

    let method_not_allowed =
        render_service_api_endpoint_response(&snapshot, "DELETE", "/v1/messages/send", "");
    assert_eq!(method_not_allowed.status_code, 405);
    let method_not_allowed_payload = parse_error_envelope(method_not_allowed.body.as_str());
    assert_eq!(method_not_allowed_payload.error, "method-not-allowed");
    assert_eq!(
        method_not_allowed_payload.reason_code,
        "service_api_method_not_allowed"
    );
    assert!(method_not_allowed_payload
        .message
        .contains("method not allowed"));

    let not_found = render_service_api_endpoint_response(&snapshot, "GET", "/v1/nope", "");
    assert_eq!(not_found.status_code, 404);
    let not_found_payload = parse_error_envelope(not_found.body.as_str());
    assert_eq!(not_found_payload.error, "not-found");
    assert_eq!(not_found_payload.reason_code, "service_api_route_not_found");
    assert!(not_found_payload.message.contains("not found"));

    let baseline_config = ServiceApiEndpointConfig {
        bind_addr: "127.0.0.1:0".to_owned(),
        max_requests: 1,
        idle_timeout_ms: 1,
        body_limit_bytes: 1,
        concurrency_limit: 1,
        rate_limit_per_second: 1,
    };

    let mut max_requests_zero = baseline_config.clone();
    max_requests_zero.max_requests = 0;
    let max_requests_error = serve_service_api_endpoint(&max_requests_zero, &snapshot)
        .expect_err("max_requests=0 must fail closed");
    assert_eq!(
        max_requests_error,
        "service api max requests must be greater than zero"
    );

    let mut idle_timeout_zero = baseline_config.clone();
    idle_timeout_zero.idle_timeout_ms = 0;
    let idle_timeout_error = serve_service_api_endpoint(&idle_timeout_zero, &snapshot)
        .expect_err("idle_timeout_ms=0 must fail closed");
    assert_eq!(
        idle_timeout_error,
        "service api idle timeout must be greater than zero"
    );

    let mut body_limit_zero = baseline_config.clone();
    body_limit_zero.body_limit_bytes = 0;
    let body_limit_error = serve_service_api_endpoint(&body_limit_zero, &snapshot)
        .expect_err("body_limit_bytes=0 must fail closed");
    assert_eq!(
        body_limit_error,
        "service api body limit bytes must be greater than zero"
    );

    let mut concurrency_limit_zero = baseline_config.clone();
    concurrency_limit_zero.concurrency_limit = 0;
    let concurrency_limit_error = serve_service_api_endpoint(&concurrency_limit_zero, &snapshot)
        .expect_err("concurrency_limit=0 must fail closed");
    assert_eq!(
        concurrency_limit_error,
        "service api concurrency limit must be greater than zero"
    );

    let mut rate_limit_zero = baseline_config;
    rate_limit_zero.rate_limit_per_second = 0;
    let rate_limit_error = serve_service_api_endpoint(&rate_limit_zero, &snapshot)
        .expect_err("rate_limit_per_second=0 must fail closed");
    assert_eq!(
        rate_limit_error,
        "service api rate limit per second must be greater than zero"
    );

    let relay_entry = ServiceApiRelaySpoolEntry {
        message_id: "msg-test-relay".to_owned(),
        sender_did: Some("kamn:did:agent:sender".to_owned()),
        recipient_did: "kamn:did:agent:recipient".to_owned(),
        body: "{\"message\":\"relay\"}".to_owned(),
        queued_at_unix: 1,
    };
    let relayed = upsert_service_api_relayed_message_from_daemon(None, &relay_entry)
        .expect("daemon relay upsert should succeed without a state file");
    assert_eq!(relayed.message_id, "msg-test-relay");
    assert_eq!(relayed.status, "relayed");
}

#[test]
fn regression_service_api_payload_parse_reason_codes_fail_closed() {
    let _env = acquire_service_api_test_env();
    let syntax_error = parse_service_api_payload::<ServiceApiHealthBody>("{\"status\":\"ok\"");
    let syntax_reason = syntax_error.expect_err("invalid json syntax should fail closed");
    assert!(
        syntax_reason.starts_with("service_api_payload_json_syntax_invalid:"),
        "unexpected syntax reason marker: {syntax_reason}"
    );

    let structure_error = parse_service_api_payload::<ServiceApiHealthBody>(
        "{\"status\":\"ok\",\"runtime_mode\":\"api\"}",
    );
    let structure_reason =
        structure_error.expect_err("invalid payload structure should fail closed");
    assert!(
        structure_reason.starts_with("service_api_payload_structure_invalid:"),
        "unexpected structure reason marker: {structure_reason}"
    );
}

#[test]
fn regression_service_api_endpoint_rejects_unknown_task_and_escrow_resource_transitions() {
    // Regression: #5866
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34112".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let caller_did = "kamn:did:agent:test-client-missing-resource";

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 3,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let accept_signature =
        service_api_request_signature_for_fields(caller_did, 71, state_hash.as_str(), "");
    let accept_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/tasks/task-missing-71/accept",
        "",
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "71"),
            ("X-KAMN-Request-Signature", accept_signature.as_str()),
        ],
    );
    assert!(accept_response.contains("HTTP/1.1 404 Not Found"));
    let accept_payload = parse_error_envelope(extract_http_response_body(accept_response.as_str()));
    assert_eq!(accept_payload.error, "not-found");
    assert_eq!(accept_payload.reason_code, "service_api_route_not_found");

    let query_signature =
        service_api_request_signature_for_fields(caller_did, 72, state_hash.as_str(), "");
    let query_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "GET",
        "/v1/tasks/task-missing-71",
        "",
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "72"),
            ("X-KAMN-Request-Signature", query_signature.as_str()),
        ],
    );
    assert!(query_response.contains("HTTP/1.1 404 Not Found"));
    let query_payload = parse_error_envelope(extract_http_response_body(query_response.as_str()));
    assert_eq!(query_payload.error, "not-found");
    assert_eq!(query_payload.reason_code, "service_api_route_not_found");

    let release_signature =
        service_api_request_signature_for_fields(caller_did, 73, state_hash.as_str(), "");
    let release_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/escrow/escrow-missing-71/release",
        "",
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "73"),
            ("X-KAMN-Request-Signature", release_signature.as_str()),
        ],
    );
    assert!(release_response.contains("HTTP/1.1 404 Not Found"));
    let release_payload =
        parse_error_envelope(extract_http_response_body(release_response.as_str()));
    assert_eq!(release_payload.error, "not-found");
    assert_eq!(release_payload.reason_code, "service_api_route_not_found");

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after missing-resource regression flow"
    );
}
