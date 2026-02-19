const DOC: &str = include_str!("../../../docs/observability/contracts.md");
const RUNTIME_NETWORK_DOC: &str = include_str!("../../../docs/foundation/runtime-network.md");
const LOGGING_SRC: &str = include_str!("../src/logging.rs");
const DAEMON_PHASE_SRC: &str = include_str!("../src/runtime_orchestration/daemon_phase.rs");
const OBS_ENDPOINT_SRC: &str = include_str!("../src/observability_endpoint.rs");

#[test]
fn unit_tracing_taxonomy_required_field_vocabulary_is_documented() {
    assert!(DOC.contains("execution_id"));
    assert!(DOC.contains("runtime_mode"));
    assert!(DOC.contains("route"));
    assert!(DOC.contains("reason_code"));
    assert!(DOC.contains("transport_checkpoint_failures"));
    assert!(DOC.contains("signer_checkpoint_failures"));
    assert!(DOC.contains("commit_checkpoint_failures"));
}

#[test]
fn functional_tracing_taxonomy_declares_version_and_required_events() {
    assert!(DOC.contains("tracing_event_taxonomy_version=kamn.node.tracing-event-taxonomy.v1"));
    assert!(DOC.contains("runtime_daemon_tick_summary"));
    assert!(DOC.contains("runtime_daemon_shutdown_checkpoint_reconciliation"));
    assert!(DOC.contains("runtime_observability_endpoint_request"));
}

#[test]
fn integration_tracing_taxonomy_docs_align_with_runtime_sources() {
    assert!(LOGGING_SRC.contains("reason_code"));
    assert!(DAEMON_PHASE_SRC.contains("execution_id"));
    assert!(DAEMON_PHASE_SRC.contains("runtime_mode"));
    assert!(OBS_ENDPOINT_SRC.contains("route(\"/\", any(handle_observability_http_route))"));
    assert!(OBS_ENDPOINT_SRC.contains("runtime_mode"));
    assert!(OBS_ENDPOINT_SRC.contains("reason_code"));
    assert!(OBS_ENDPOINT_SRC.contains("transport_checkpoint_failures"));
    assert!(OBS_ENDPOINT_SRC.contains("signer_checkpoint_failures"));
    assert!(OBS_ENDPOINT_SRC.contains("commit_checkpoint_failures"));
}

#[test]
fn regression_tracing_taxonomy_declares_drift_fail_closed_reason_markers() {
    assert!(DOC.contains("runtime_tracing_taxonomy_required_field_missing:<event>:<field>"));
    assert!(DOC.contains("runtime_tracing_taxonomy_schema_drift:<event>:<field>"));
    assert!(DOC.contains("runtime_tracing_taxonomy_event_marker_missing:<event>"));
}

#[test]
fn unit_startup_logging_contract_declares_required_env_controls() {
    assert!(
        DOC.contains("startup_logging_configuration_version=kamn.node.startup-logging-config.v1")
    );
    assert!(DOC.contains("KAMN_NODE_LOG_LEVEL"));
    assert!(DOC.contains("KAMN_NODE_LOG_FORMAT"));
    assert!(DOC.contains("accepted values: `error`, `warn`, `info`, `debug`, `trace`"));
    assert!(DOC.contains("accepted values: `text`, `json`"));
}

#[test]
fn functional_startup_logging_contract_declares_runtime_mode_bootstrap_coverage() {
    assert!(DOC.contains("Runtime modes with deterministic tracing bootstrap"));
    assert!(DOC.contains("`bootstrap`"));
    assert!(DOC.contains("`full`"));
    assert!(DOC.contains("`kolme-live`"));
}

#[test]
fn integration_startup_logging_contract_docs_align_with_runtime_source_markers() {
    assert!(LOGGING_SRC.contains("KAMN_NODE_LOG_LEVEL_ENV"));
    assert!(LOGGING_SRC.contains("KAMN_NODE_LOG_FORMAT_ENV"));
    assert!(LOGGING_SRC.contains("must be one of: error,warn,info,debug,trace"));
    assert!(LOGGING_SRC.contains("must be one of: text,json"));
}

#[test]
fn regression_startup_logging_contract_declares_fail_closed_invalid_config_markers() {
    assert!(DOC.contains("ConfigError::InvalidLogConfig"));
    assert!(DOC.contains("KAMN_NODE_LOG_LEVEL must be one of: error,warn,info,debug,trace"));
    assert!(DOC.contains("KAMN_NODE_LOG_FORMAT must be one of: text,json"));
}

#[test]
fn unit_runtime_network_observability_tls_contract_declares_required_env_markers() {
    assert!(RUNTIME_NETWORK_DOC.contains("KAMN_OBSERVABILITY_ENDPOINT_TLS_MODE=disabled|require"));
    assert!(RUNTIME_NETWORK_DOC.contains("KAMN_OBSERVABILITY_ENDPOINT_TLS_CERT_FILE"));
    assert!(RUNTIME_NETWORK_DOC.contains("KAMN_OBSERVABILITY_ENDPOINT_TLS_KEY_FILE"));
}

#[test]
fn functional_runtime_network_observability_tls_contract_declares_negative_matrix_reason_codes() {
    assert!(RUNTIME_NETWORK_DOC.contains("observability_tls_negative_matrix_status=verified"));
    assert!(RUNTIME_NETWORK_DOC.contains(
        "observability_tls_negative_matrix_reason_codes_csv=observability_endpoint_tls_certificate_file_read_failed,observability_endpoint_tls_key_file_parse_failed,observability_endpoint_tls_mode_invalid,observability_endpoint_tls_plain_http_handshake_rejected"
    ));
}

#[test]
fn integration_runtime_network_observability_tls_contract_aligns_with_endpoint_source() {
    assert!(OBS_ENDPOINT_SRC.contains("KAMN_OBSERVABILITY_ENDPOINT_TLS_MODE"));
    assert!(OBS_ENDPOINT_SRC.contains("KAMN_OBSERVABILITY_ENDPOINT_TLS_CERT_FILE"));
    assert!(OBS_ENDPOINT_SRC.contains("KAMN_OBSERVABILITY_ENDPOINT_TLS_KEY_FILE"));
    assert!(OBS_ENDPOINT_SRC.contains("observability endpoint tls certificate file read failed"));
    assert!(OBS_ENDPOINT_SRC.contains("observability endpoint tls key file parse failed"));
    assert!(OBS_ENDPOINT_SRC.contains("observability endpoint tls mode is invalid"));
}

#[test]
fn regression_runtime_network_observability_tls_contract_declares_policy_drift_marker() {
    assert!(RUNTIME_NETWORK_DOC
        .contains("runtime_observability_policy_tls_negative_matrix_reason_codes_csv_mismatch"));
}

#[test]
fn unit_observability_route_parity_contract_declares_required_matrix_markers() {
    assert!(DOC.contains(
        "observability_route_parity_matrix_version=kamn.runtime.observability.route-parity.v1"
    ));
    assert!(DOC.contains("GET /metrics -> 200 text/plain; version=0.0.4"));
    assert!(DOC.contains("GET /healthz -> 200 application/json"));
    assert!(DOC.contains("GET /readyz -> 200 application/json"));
    assert!(DOC.contains("GET /metrics.stream -> 200 application/x-ndjson"));
}

#[test]
fn functional_observability_route_parity_contract_declares_fail_closed_rows() {
    assert!(DOC.contains("GET /unknown -> 404 text/plain; charset=utf-8"));
    assert!(DOC.contains("POST /metrics -> 404 text/plain; charset=utf-8"));
    assert!(DOC.contains("route_parity_checkpoint_status=verified"));
    assert!(DOC.contains("fail_closed_checkpoint_status=verified"));
    assert!(DOC.contains("route_class_coverage_status=verified"));
}

#[test]
fn integration_observability_route_parity_contract_docs_align_with_endpoint_route_constants() {
    assert!(OBS_ENDPOINT_SRC.contains("DEFAULT_OBSERVABILITY_ENDPOINT_METRICS_PATH"));
    assert!(OBS_ENDPOINT_SRC.contains("DEFAULT_OBSERVABILITY_ENDPOINT_HEALTH_PATH"));
    assert!(OBS_ENDPOINT_SRC.contains("DEFAULT_OBSERVABILITY_ENDPOINT_READINESS_PATH"));
    assert!(OBS_ENDPOINT_SRC.contains("DEFAULT_OBSERVABILITY_ENDPOINT_STREAM_PATH"));
}

#[test]
fn regression_observability_route_parity_contract_declares_drift_reason_markers() {
    assert!(DOC.contains(
        "service_api_observability_route_compatibility_policy_matrix_row_missing:<row_id>"
    ));
    assert!(DOC.contains(
        "service_api_observability_route_compatibility_policy_matrix_row_route_mismatch:<row_id>"
    ));
    assert!(DOC.contains(
        "service_api_observability_route_compatibility_policy_matrix_row_status_mismatch:<row_id>"
    ));
    assert!(DOC.contains(
        "service_api_observability_route_compatibility_policy_matrix_row_content_type_mismatch:<row_id>"
    ));
    assert!(DOC.contains(
        "service_api_observability_route_compatibility_policy_marker_missing:route_parity_checkpoint_status"
    ));
}
