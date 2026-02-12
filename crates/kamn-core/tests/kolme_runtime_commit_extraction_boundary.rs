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
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_block_fallback_policy_error_to_malformed("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_endpoint_policy_error_to_malformed("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_notification_policy_error_to_malformed("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_provider_response_policy_error_to_malformed("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_broadcast_payload_policy_error_to_malformed("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_endpoint_policy_error("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_websocket_policy_error("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_http_response_policy_error("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_codec_error_to_invalid_request("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_codec_error_to_malformed_response("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_provider_outcome("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_provider_receipt("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_provider_error("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_transport_io_error("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_transport_io_classification_to_provider_error("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_provider_outcome_policy_error_to_malformed("));
}

#[test]
fn regression_runtime_commit_extraction_boundary_keeps_direct_helper_delegation() {
    // Regression: #1824
    assert!(RUNTIME_COMMIT_SRC.contains("parse_kolme_provider_finality_receipt("));
    assert!(
        RUNTIME_COMMIT_SRC.contains("require_kolme_commit_id_matches_expected_txhash_contract(")
    );
    assert!(RUNTIME_COMMIT_SRC.contains("lifecycle_state_for_finality_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("lifecycle_state_label_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("commit_finality_label_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_terminal_receipt_finality_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_poll_attempt_budget_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_runtime_commit_id_request_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("deterministic_runtime_commit_idempotency_key_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("deterministic_runtime_commit_id_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("txhash_from_kolme_commit_id("));
    assert!(RUNTIME_COMMIT_SRC.contains("parse_kolme_http_endpoint("));
    assert!(RUNTIME_COMMIT_SRC.contains("parse_kolme_websocket_endpoint("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_live_provider_base_url_input_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_live_provider_submit_path_input_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_expected_provider_input_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("compose_kolme_notifications_websocket_url("));
    assert!(RUNTIME_COMMIT_SRC.contains("validate_kolme_block_path_template("));
    assert!(RUNTIME_COMMIT_SRC.contains("render_kolme_block_path("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_block_lookup_height_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("parse_kolme_provider_block_fallback_response_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_block_fallback_base_url_input_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_block_fallback_provider_input_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_block_fallback_lookup_budget_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("classify_kolme_tls_failure_reason("));
    assert!(RUNTIME_COMMIT_SRC.contains("normalize_kolme_broadcast_payload_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("validate_kolme_websocket_handshake_response("));
    assert!(RUNTIME_COMMIT_SRC.contains("resolve_kolme_tls_ca_file_env_result_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("parse_kolme_authorization_header_value("));
    assert!(RUNTIME_COMMIT_SRC.contains("validate_kolme_lookup_window("));
    assert!(RUNTIME_COMMIT_SRC.contains("resolve_kolme_lookup_upper_bound("));
    assert!(RUNTIME_COMMIT_SRC.contains("validate_kolme_lookup_txhash_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("compose_kolme_block_fallback_unresolved_reason_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("project_kolme_finalized_block_txhash_receipt_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("project_kolme_failed_block_txhash_receipt_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("validate_kolme_block_identity("));
    assert!(RUNTIME_COMMIT_SRC.contains("compose_kolme_finality_status_path("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_finality_base_url_input_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_finality_status_path_input_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_notifications_provider_input_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_notifications_reconnect_budget_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("parse_kolme_notification_event_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("notification_event_to_kolme_provider_receipt_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("parse_kolme_live_runtime_provider_outcome_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("parse_kolme_http_response_body("));
    assert!(RUNTIME_COMMIT_SRC.contains("find_kolme_http_header_boundary("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_websocket_timeout_seconds_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("validate_kolme_provider_receipt_identity_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("require_kolme_final_receipt_finality_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("KamnKolmeApiNextNonceRequest::new("));
    assert!(RUNTIME_COMMIT_SRC.contains("KamnKolmeApiBroadcastResponse::parse_json("));
    assert!(RUNTIME_COMMIT_SRC.contains("KolmeRuntimeCommitTransportErrorKind::Timeout"));
    assert!(RUNTIME_COMMIT_SRC.contains("KolmeRuntimeCommitTransportErrorKind::MalformedResponse"));
    assert!(RUNTIME_COMMIT_SRC.contains("classify_kolme_transport_io_error(&error)"));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_http_transport_timeout_seconds_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_runtime_provider_input_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_provider_hint_input_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_runtime_operation_id_input_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_runtime_state_root_input_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_runtime_payload_hash_input_contract("));
    assert!(RUNTIME_COMMIT_SRC
        .contains("are_kolme_runtime_commit_request_fields_single_line_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_receipt_provider_input_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_receipt_commit_id_input_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_transport_idempotency_key_input_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("is_kolme_valid_transport_wire_payload_input_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("normalize_kolme_broadcast_submit_path_input_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("impl From<KamnKolmeTransportIoClassification>"));
}
