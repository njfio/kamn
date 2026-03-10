use crate::support::{assert_markers, DOC};

const OUTPUT_MODE_SCOPE_MARKERS: &[&str] = &[
    "## Scope Delivered",
    "--output text",
    "--output json",
    "ConfigError::InvalidOutputMode",
    "--profile local-listener",
    "ConfigError::InvalidNodeProfile",
    "--diagnostics snapshot",
    "ConfigError::InvalidDiagnosticsMode",
    "--runtime-mode planning",
    "--runtime-mode recovery-check",
    "--runtime-mode daemon",
    "--runtime-mode kolme-live",
    "--kolme-live-base-url",
    "--kolme-live-provider-hint",
    "--kolme-live-signing-profile",
    "ConfigError::InvalidRuntimeMode",
    "ConfigError::InvalidDaemonControlArgument",
    "ConfigError::InvalidDaemonLifecycleEvent",
    "ConfigError::RuntimeDaemonLifecycle",
    "ConfigError::InvalidKolmeLiveProviderHint",
    "ConfigError::InvalidKolmeLiveSigningProfile",
    "ConfigError::RuntimeKolmeLive",
];
const DETERMINISTIC_JSON_MARKERS: &[&str] = &[
    "JSON output is deterministic and includes:",
    "runtime_mode",
    "diagnostics_mode",
    "profile",
    "component_count",
    "planning_candidate_count",
    "planning_scheduled_candidate_ids",
    "recovery_expected_state_version",
    "recovery_attempt_count",
    "recovery_decisions",
    "daemon_max_ticks",
    "daemon_executed_ticks",
    "daemon_completion_reason",
    "daemon_observability_latency_p50_ms",
    "daemon_observability_latency_p99_ms",
    "daemon_observability_throughput_tps",
    "daemon_observability_error_rate_bps",
    "daemon_observability_availability_bps",
    "daemon_observability_health",
    "daemon_observability_alert_count",
    "daemon_peer_lifecycle_final_state",
    "daemon_peer_lifecycle_applied_events",
    "kolme_live_provider_client_contract",
    "kolme_live_base_url",
    "kolme_live_provider_hint",
    "kolme_live_signing_profile",
    "kolme_live_signer_profile_selector_env",
    "kolme_live_signer_profile",
    "kolme_live_signer_key_source",
    "kolme_live_signer_private_key_env",
    "kolme_live_execution_status",
    "kolme_live_observability_latency_p50_ms",
    "kolme_live_observability_latency_p99_ms",
    "kolme_live_observability_throughput_tps",
    "kolme_live_observability_error_rate_bps",
    "kolme_live_observability_availability_bps",
    "kolme_live_observability_health",
    "kolme_live_observability_alert_count",
    "sync_mode",
    "components",
];
const LOCAL_PROFILE_MARKERS: &[&str] = &[
    "## Local Profile Rules",
    "chain_id`: `kamn-localnet`",
    "storage_dir`: role-scoped",
    "Explicit CLI flags override profile defaults",
];
const DIAGNOSTICS_MARKERS: &[&str] = &[
    "## Diagnostics Snapshot Rules",
    "`basic` (default)",
    "`snapshot`",
    "component_count",
];
const PLANNING_MARKERS: &[&str] = &[
    "## Runtime Planning Rules",
    "`planning`",
    "--expected-state-hash",
    "--proposal <id|sender-did|nonce|state-hash>",
    "Duplicate candidate IDs and stale state hashes are rejected",
];
const RECOVERY_MARKERS: &[&str] = &[
    "## Recovery Check Rules",
    "`recovery-check`",
    "--expected-state-version",
    "--rejoin-attempt <node-id|state-version|state-hash|resume-token>",
    "Replay resume tokens and version/hash mismatch scenarios are rejected",
];
const RUNTIME_MODE_COMMAND_MARKERS: &[&str] = &[
    "`kamn-node --role processor --runtime-mode planning`",
    "`kamn-node --role processor --runtime-mode planning --expected-state-hash state-1 --proposal tx-1|kamn:did:agent:aaa|1|state-1`",
    "`kamn-node --role processor --runtime-mode recovery-check`",
    "`kamn-node --role processor --runtime-mode daemon`",
];

#[test]
fn doc_contains_output_mode_scope_and_rules() {
    assert_markers(DOC, OUTPUT_MODE_SCOPE_MARKERS, "node runtime CLI output mode scope rules");
}

#[test]
fn doc_contains_deterministic_json_fields() {
    assert_markers(DOC, DETERMINISTIC_JSON_MARKERS, "node runtime CLI deterministic JSON fields");
}

#[test]
fn doc_contains_local_profile_rules() {
    assert_markers(DOC, LOCAL_PROFILE_MARKERS, "node runtime CLI local profile rules");
}

#[test]
fn doc_contains_diagnostics_snapshot_rules() {
    assert_markers(DOC, DIAGNOSTICS_MARKERS, "node runtime CLI diagnostics rules");
}

#[test]
fn doc_contains_runtime_planning_rules() {
    assert_markers(DOC, PLANNING_MARKERS, "node runtime CLI planning rules");
}

#[test]
fn doc_contains_runtime_recovery_check_rules() {
    assert_markers(DOC, RECOVERY_MARKERS, "node runtime CLI recovery-check rules");
}

#[test]
fn doc_contains_runtime_mode_command_examples() {
    assert_markers(DOC, RUNTIME_MODE_COMMAND_MARKERS, "node runtime CLI runtime-mode examples");
}
