use std::fs;

pub(super) const ROOT_FILE: &str = "src/main_tests/service_api_endpoint_tests.rs";
pub(super) const WEBSOCKET_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests.rs";
pub(super) const AUTH_SCOPE_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/auth_scope_contract_tests.rs";
pub(super) const AUTH_BINDING_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/auth_scope_contract_tests/auth_binding_contract_tests.rs";
pub(super) const ROUTE_SCOPE_POLICY_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/auth_scope_contract_tests/route_scope_policy_contract_tests.rs";
pub(super) const LEGACY_SIGNATURE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/auth_scope_contract_tests/legacy_signature_contract_tests.rs";
pub(super) const ROUTE_RENDER_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/route_render_contract_tests.rs";
pub(super) const ROUTE_RESPONSE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/route_render_contract_tests/route_response_contract_tests.rs";
pub(super) const ROUTE_METRICS_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/route_render_contract_tests/route_metrics_contract_tests.rs";
pub(super) const MESSAGE_PERSISTENCE_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/message_persistence_contract_tests.rs";
pub(super) const MESSAGE_RESTART_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/message_persistence_contract_tests/message_restart_contract_tests.rs";
pub(super) const MESSAGE_RUNTIME_EVIDENCE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/message_persistence_contract_tests/message_runtime_evidence_contract_tests.rs";
pub(super) const CHANNEL_AGENT_DIRECTORY_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/channel_agent_directory_contract_tests.rs";
pub(super) const CHANNEL_STATE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/channel_agent_directory_contract_tests/channel_state_contract_tests.rs";
pub(super) const AGENT_PROFILE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/channel_agent_directory_contract_tests/agent_profile_contract_tests.rs";
pub(super) const AGENT_REGISTRY_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/channel_agent_directory_contract_tests/agent_registry_contract_tests.rs";
pub(super) const TASK_ESCROW_PERSISTENCE_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/task_escrow_persistence_contract_tests.rs";
pub(super) const TASK_ESCROW_ROUTES_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/task_escrow_persistence_contract_tests/task_escrow_routes_contract_tests.rs";
pub(super) const TASK_ESCROW_RESTART_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/task_escrow_persistence_contract_tests/task_escrow_restart_contract_tests.rs";
pub(super) const CONTENT_LIFECYCLE_RESTART_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/content_lifecycle_restart_contract_tests.rs";
pub(super) const CONTENT_LIFECYCLE_RESTART_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/content_lifecycle_restart_contract_tests/content_lifecycle_restart_contract_tests.rs";
pub(super) const BRIDGE_PERSISTENCE_RESTART_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/bridge_persistence_restart_contract_tests.rs";
pub(super) const BRIDGE_PERSISTENCE_RESTART_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/bridge_persistence_restart_contract_tests/bridge_persistence_restart_contract_tests.rs";
pub(super) const MAILBOX_RELAY_DELIVERY_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests.rs";
pub(super) const RECIPIENT_MAILBOX_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests/recipient_mailbox_contract_tests.rs";
pub(super) const RELAY_DELIVERY_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests/relay_delivery_contract_tests.rs";
pub(super) const RELAY_DID_REJECTION_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests/relay_did_rejection_contract_tests.rs";
pub(super) const RELAY_STATUS_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests/relay_status_contract_tests.rs";
pub(super) const MAILBOX_RELAY_SUPPORT_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests/support.rs";
pub(super) const MAILBOX_RELAY_STATE_SUPPORT_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests/state_support.rs";
pub(super) const INGRESS_GUARD_LIFECYCLE_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests.rs";
pub(super) const INGRESS_BUDGET_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests/ingress_budget_contract_tests.rs";
pub(super) const SENDER_ANTI_SPAM_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests/sender_anti_spam_contract_tests.rs";
pub(super) const REPLAY_GUARD_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests/replay_guard_contract_tests.rs";
pub(super) const CONCURRENCY_GUARD_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests/concurrency_guard_contract_tests.rs";
pub(super) const LIFECYCLE_PROJECTION_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests/lifecycle_projection_contract_tests.rs";
pub(super) const TRANSPORT_SURFACE_OBSERVABILITY_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/transport_surface_observability_contract_tests.rs";
pub(super) const ROUTE_TLS_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/transport_surface_observability_contract_tests/route_tls_contract_tests.rs";
pub(super) const HTTP_CONNECTION_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/transport_surface_observability_contract_tests/http_connection_contract_tests.rs";
pub(super) const OBSERVABILITY_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/transport_surface_observability_contract_tests/observability_contract_tests.rs";
pub(super) const TRANSPORT_SUPPORT_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/transport_surface_observability_contract_tests/support.rs";
pub(super) const SHARED_SUPPORT_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/shared_support.rs";
pub(super) const AUTH_FIXTURE_SUPPORT_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/shared_support/auth_fixture_support.rs";
pub(super) const ROUTE_SCOPE_SUPPORT_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/shared_support/route_scope_support.rs";
pub(super) const HTTP_TRANSPORT_SUPPORT_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/shared_support/http_transport_support.rs";
pub(super) const TLS_TRANSPORT_SUPPORT_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/shared_support/tls_transport_support.rs";
pub(super) const RESPONSE_SUPPORT_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/shared_support/response_support.rs";
pub(super) const ENV_SUPPORT_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/shared_support/env_support.rs";
pub(super) const RESIDUAL_ROOT_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/residual_root_contract_tests.rs";
pub(super) const ENV_LOCK_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/residual_root_contract_tests/env_lock_contract_tests.rs";
pub(super) const SERDE_PAYLOAD_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/residual_root_contract_tests/serde_payload_contract_tests.rs";
pub(super) const ERROR_ENVELOPE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/residual_root_contract_tests/error_envelope_contract_tests.rs";
pub(super) const PAYLOAD_PARSE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/residual_root_contract_tests/payload_parse_contract_tests.rs";
pub(super) const MISSING_RESOURCE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/residual_root_contract_tests/missing_resource_contract_tests.rs";
pub(super) const ROOT_STAGED_MAX_LINES: usize = 200;

pub(super) fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

pub(super) fn line_count(path: &str) -> usize {
    read_repo_file(path).lines().count()
}

pub(super) fn assert_contains_markers(source: &str, markers: &[&str], label: &str) {
    for marker in markers {
        assert!(
            source.contains(marker),
            "{label} should include moved marker: {marker}"
        );
    }
}
