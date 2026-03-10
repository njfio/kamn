use super::{
    build_bootstrap_report, build_kolme_live_direct_signed_wire_payload,
    build_kolme_live_managed_signing_key, build_kolme_live_request,
    build_kolme_live_signer_adapter, build_runtime_observability_snapshot,
    build_service_api_snapshot, capture_test_logs,
    classify_full_bootstrap_component_contract_violation,
    classify_full_supervisor_stop_contract_violation,
    classify_kolme_live_signer_key_source_policy_violation,
    classify_production_transport_profile_violation, classify_service_api_endpoint_runtime_path,
    encode_kolme_hex_lower, enforce_kolme_live_signer_contract_policy,
    enforce_kolme_live_signer_key_source_policy, enforce_kolme_live_signer_preflight, execute,
    parse_args, render_bootstrap_report, render_kolme_live_native_direct_message,
    render_log_event_line, render_observability_endpoint_response,
    render_service_api_endpoint_response, reset_cached_log_config_for_tests,
    resolve_kolme_live_allow_local_signer_testing_override,
    resolve_kolme_live_managed_signer_required_marker, resolve_kolme_live_nonce,
    resolve_kolme_live_signer_private_key_env_name, resolve_log_config_from_inputs,
    select_runtime_transport_profile_for_runtime_mode, serve_observability_endpoint,
    serve_service_api_endpoint, should_skip_observability_endpoint_for_full_supervisor,
    sign_kolme_live_managed_external_message, validate_full_supervisor_stop_contract,
    DiagnosticsMode, KolmeForkSecp256k1SignerAdapter, LocalProfile, NodeBootstrapReport,
    NodeLogConfig, NodeLogFormat, NodeLogLevel, ObservabilityEndpointConfig, OutputMode,
    RuntimeExecutionBundle, RuntimeMode, ServiceApiEndpointConfig, ServiceApiEndpointRuntimePath,
    KAMN_NODE_LOG_FORMAT_ENV, KAMN_NODE_LOG_LEVEL_ENV,
};
use constants::{
    TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE, TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE_SECONDARY,
    TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX, TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY,
};
use kamn_core::{
    bootstrap, ConfigError, KolmeRuntimeCommitHttpTransport, KolmeRuntimeCommitRequest, NodeConfig,
    NodeRole, SignerProviderHandshakeMatrix, SyncMode,
};
use std::env;
use std::time::{Duration, Instant};

use support::{
    extract_json_string_field, lock_signer_env_guard, log_env_lock, managed_signer_public_key_hex,
    request_body, signer_env_lock, spawn_kolme_live_mock_server, EnvVarGuard, MockHttpReply,
};

// main_tests structural budget shell only; keep domain tests in src/main_tests/*.rs
mod async_runtime_contract_tests;
mod cli_contract_tests;
mod constants;
mod core_behavior_tests;
mod daemon_tests;
mod observability_endpoint_tests;
mod report_tests;
mod runtime_tests;
mod service_api_endpoint_tests;
mod service_api_endpoint_tests_split_contract;
mod signer_tests;
mod support;
