const RUNTIME_COMMIT_SRC: &str = include_str!("../src/kolme_runtime_commit.rs");

#[test]
fn unit_runtime_commit_extraction_boundary_removes_local_finality_glue_wrappers() {
    assert!(!RUNTIME_COMMIT_SRC.contains("fn parse_receipt_finality("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_extracted_receipt_finality("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn lifecycle_state_for_finality("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn lifecycle_state_label("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn commit_finality_label("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn deterministic_idempotency_key("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn deterministic_commit_id("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn deterministic_backend_commit_id("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn txhash_from_commit_id("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn parse_http_endpoint("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn parse_websocket_endpoint("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn compose_notifications_websocket_url("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn validate_block_path_template("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn render_block_path("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn parse_block_fallback_response("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn parse_kolme_fork_block_fallback_response("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn classify_tls_failure_reason("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn normalize_kolme_broadcast_payload("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn validate_websocket_handshake_response("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_tls_policy_error("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_transport_request_policy_error("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_block_scan_policy_error_to_unavailable("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_block_scan_policy_error_to_malformed("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_lookup_window_error("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_endpoint_policy_error_to_malformed("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_notification_policy_error_to_malformed("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_provider_response_policy_error_to_malformed("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_broadcast_payload_policy_error_to_malformed("));
}

#[test]
fn regression_runtime_commit_extraction_boundary_keeps_direct_helper_delegation() {
    // Regression: #1806
    assert!(RUNTIME_COMMIT_SRC.contains("parse_kolme_commit_receipt_finality("));
    assert!(RUNTIME_COMMIT_SRC.contains("commit_finality_from_receipt_finality_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("lifecycle_state_for_finality_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("lifecycle_state_label_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("commit_finality_label_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("deterministic_runtime_commit_idempotency_key_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("deterministic_runtime_commit_id_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("deterministic_kolme_backend_commit_id("));
    assert!(RUNTIME_COMMIT_SRC.contains("txhash_from_kolme_commit_id("));
    assert!(RUNTIME_COMMIT_SRC.contains("parse_kolme_http_endpoint("));
    assert!(RUNTIME_COMMIT_SRC.contains("parse_kolme_websocket_endpoint("));
    assert!(RUNTIME_COMMIT_SRC.contains("compose_kolme_notifications_websocket_url("));
    assert!(RUNTIME_COMMIT_SRC.contains("validate_kolme_block_path_template("));
    assert!(RUNTIME_COMMIT_SRC.contains("render_kolme_block_path("));
    assert!(RUNTIME_COMMIT_SRC.contains("parse_kolme_block_fallback_response_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("parse_kolme_fork_block_fallback_response_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("classify_kolme_tls_failure_reason("));
    assert!(RUNTIME_COMMIT_SRC.contains("normalize_kolme_broadcast_payload_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("validate_kolme_websocket_handshake_response("));
    assert!(RUNTIME_COMMIT_SRC.contains("parse_kolme_tls_ca_file_env_value("));
    assert!(RUNTIME_COMMIT_SRC.contains("parse_kolme_authorization_header_value("));
    assert!(RUNTIME_COMMIT_SRC.contains("validate_kolme_lookup_window("));
    assert!(RUNTIME_COMMIT_SRC.contains("validate_kolme_block_identity("));
    assert!(RUNTIME_COMMIT_SRC.contains("compose_kolme_finality_status_path("));
    assert!(RUNTIME_COMMIT_SRC.contains("parse_kolme_provider_response_fields("));
    assert!(RUNTIME_COMMIT_SRC.contains("parse_kolme_notification_event_contract("));
}
