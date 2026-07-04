use super::super::*;
use super::support::{
    build_content_snapshot, expire_content, query_content, query_missing_content, read_state_json,
    register_content, set_state_file_env, tombstone_content, unique_named_state_file,
};

#[test]
fn integration_service_api_endpoint_persists_content_lifecycle_state_across_restart() {
    let _env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-content-restart-state");
    let _state_file_guard = set_state_file_env(state_file.as_path());
    let caller_did = "kamn:did:agent:test-client-content-restart";
    let content_id = assert_initial_content_lifecycle_phase(caller_did);
    assert_restart_content_lifecycle_phase(caller_did, content_id, state_file.as_path());
    let _ = fs::remove_file(state_file);
}

fn assert_initial_content_lifecycle_phase(caller_did: &str) -> String {
    let first_snapshot = build_content_snapshot("127.0.0.1:34113");
    let bind_addr = reserve_loopback_addr();
    let registered = register_content(
        &first_snapshot,
        bind_addr.as_str(),
        caller_did,
        91,
        r#"{"content":"restart-content-check"}"#,
    );
    let content_id = registered["content_id"]
        .as_str()
        .expect("content id should be string");
    let expired = expire_content(
        &first_snapshot,
        bind_addr.as_str(),
        caller_did,
        92,
        content_id,
    );
    assert_registered_and_expired_payloads(&registered, &expired, content_id);
    content_id.to_owned()
}

fn assert_registered_and_expired_payloads(registered: &Value, expired: &Value, content_id: &str) {
    assert_eq!(registered["retention_class"], "standard");
    assert_eq!(registered["lifecycle_state"], "retained");
    assert_eq!(registered["redaction_status"], "none");
    assert_eq!(expired["content_id"], content_id);
    assert_eq!(expired["lifecycle_state"], "expired");
    assert_eq!(expired["redaction_status"], "none");
}

fn assert_restart_content_lifecycle_phase(
    caller_did: &str,
    content_id: String,
    state_file: &std::path::Path,
) {
    let restart_snapshot = build_content_snapshot("127.0.0.1:34114");
    let restart_bind_addr = reserve_loopback_addr();
    let queried = query_content(
        &restart_snapshot,
        restart_bind_addr.as_str(),
        caller_did,
        93,
        content_id.as_str(),
    );
    let tombstoned = tombstone_content(
        &restart_snapshot,
        restart_bind_addr.as_str(),
        caller_did,
        94,
        content_id.as_str(),
    );
    let queried_after = query_content(
        &restart_snapshot,
        restart_bind_addr.as_str(),
        caller_did,
        95,
        content_id.as_str(),
    );
    let missing = query_missing_content(
        &restart_snapshot,
        restart_bind_addr.as_str(),
        "kamn:did:agent:test-client-content-missing-restart",
        96,
        "content-missing-96",
    );
    let state_json = read_state_json(state_file);
    assert_restart_payloads(
        &queried,
        &tombstoned,
        &queried_after,
        &missing,
        &state_json,
        content_id.as_str(),
    );
}

fn assert_restart_payloads(
    queried: &Value,
    tombstoned: &Value,
    queried_after: &Value,
    missing: &ServiceApiErrorEnvelope,
    state_json: &Value,
    content_id: &str,
) {
    assert_eq!(queried["content_id"], content_id);
    assert_eq!(queried["lifecycle_state"], "expired");
    assert_eq!(queried["redaction_status"], "none");
    assert_eq!(tombstoned["content_id"], content_id);
    assert_eq!(tombstoned["lifecycle_state"], "tombstoned");
    assert_eq!(tombstoned["redaction_status"], "redacted");
    assert_eq!(queried_after["content_id"], content_id);
    assert_eq!(queried_after["lifecycle_state"], "tombstoned");
    assert_eq!(queried_after["redaction_status"], "redacted");
    assert_eq!(missing.error, "not-found");
    assert_eq!(missing.reason_code, "service_api_route_not_found");
    assert_eq!(
        state_json["contents"][content_id]["lifecycle_state"],
        "tombstoned"
    );
    assert_eq!(
        state_json["contents"][content_id]["redaction_status"],
        "redacted"
    );
}
