const DOC: &str = include_str!("../../../docs/observability/contracts.md");
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
