use super::super::*;
use super::support::{
    build_message_snapshot, query_persisted_message, send_persisted_message,
    unique_named_state_file,
};

#[test]
fn integration_service_api_endpoint_persists_message_state_across_restart_without_explicit_state_file_env(
) {
    let _env = acquire_service_api_test_env();
    let snapshot = build_message_snapshot("127.0.0.1:34079");
    let bind_addr = reserve_loopback_addr();
    let send_payload = send_persisted_message(
        &snapshot,
        bind_addr.as_str(),
        "kamn:did:agent:test-client-persist-default",
        1,
        r#"{"message":"durable-store-default-check"}"#,
    );
    let query_path = format!("/v1/messages/{}", send_payload.message_id);
    let query_payload = query_persisted_message(
        &snapshot,
        bind_addr.as_str(),
        "kamn:did:agent:test-client-persist-default",
        2,
        query_path.as_str(),
    );
    assert_eq!(query_payload.message_id, send_payload.message_id);
    assert_eq!(query_payload.status, "created");
}

#[test]
fn integration_service_api_endpoint_persists_message_state_across_restart() {
    let _env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-state");
    let state_file_str = state_file.to_string_lossy().to_string();
    let _state_file_guard =
        EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(state_file_str.as_str()));
    let snapshot = build_message_snapshot("127.0.0.1:34080");
    let bind_addr = reserve_loopback_addr();
    let send_payload = send_persisted_message(
        &snapshot,
        bind_addr.as_str(),
        "kamn:did:agent:test-client-persist",
        1,
        r#"{"message":"durable-store-check"}"#,
    );
    let query_path = format!("/v1/messages/{}", send_payload.message_id);
    let query_payload = query_persisted_message(
        &snapshot,
        bind_addr.as_str(),
        "kamn:did:agent:test-client-persist",
        2,
        query_path.as_str(),
    );
    assert_eq!(query_payload.message_id, send_payload.message_id);
    assert_eq!(query_payload.status, "created");
    let _ = fs::remove_file(state_file);
}
