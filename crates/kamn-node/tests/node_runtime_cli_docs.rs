const DOC: &str = include_str!("../../../docs/foundation/node-runtime-cli.md");

#[test]
fn doc_contains_output_mode_scope_and_rules() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("--output text"));
    assert!(DOC.contains("--output json"));
    assert!(DOC.contains("ConfigError::InvalidOutputMode"));
    assert!(DOC.contains("--profile local-listener"));
    assert!(DOC.contains("ConfigError::InvalidNodeProfile"));
    assert!(DOC.contains("--diagnostics snapshot"));
    assert!(DOC.contains("ConfigError::InvalidDiagnosticsMode"));
    assert!(DOC.contains("--runtime-mode planning"));
    assert!(DOC.contains("--runtime-mode recovery-check"));
    assert!(DOC.contains("--runtime-mode daemon"));
    assert!(DOC.contains("ConfigError::InvalidRuntimeMode"));
    assert!(DOC.contains("ConfigError::InvalidDaemonControlArgument"));
    assert!(DOC.contains("ConfigError::InvalidDaemonLifecycleEvent"));
    assert!(DOC.contains("ConfigError::RuntimeDaemonLifecycle"));
}

#[test]
fn doc_contains_deterministic_json_fields() {
    assert!(DOC.contains("JSON output is deterministic and includes:"));
    assert!(DOC.contains("runtime_mode"));
    assert!(DOC.contains("diagnostics_mode"));
    assert!(DOC.contains("profile"));
    assert!(DOC.contains("component_count"));
    assert!(DOC.contains("planning_candidate_count"));
    assert!(DOC.contains("planning_scheduled_candidate_ids"));
    assert!(DOC.contains("recovery_expected_state_version"));
    assert!(DOC.contains("recovery_attempt_count"));
    assert!(DOC.contains("recovery_decisions"));
    assert!(DOC.contains("daemon_max_ticks"));
    assert!(DOC.contains("daemon_executed_ticks"));
    assert!(DOC.contains("daemon_completion_reason"));
    assert!(DOC.contains("daemon_peer_lifecycle_final_state"));
    assert!(DOC.contains("daemon_peer_lifecycle_applied_events"));
    assert!(DOC.contains("sync_mode"));
    assert!(DOC.contains("components"));
}

#[test]
fn doc_contains_local_profile_rules() {
    assert!(DOC.contains("## Local Profile Rules"));
    assert!(DOC.contains("chain_id`: `kamn-localnet`"));
    assert!(DOC.contains("storage_dir`: role-scoped"));
    assert!(DOC.contains("Explicit CLI flags override profile defaults"));
}

#[test]
fn doc_contains_diagnostics_snapshot_rules() {
    assert!(DOC.contains("## Diagnostics Snapshot Rules"));
    assert!(DOC.contains("`basic` (default)"));
    assert!(DOC.contains("`snapshot`"));
    assert!(DOC.contains("component_count"));
}

#[test]
fn doc_contains_runtime_planning_rules() {
    assert!(DOC.contains("## Runtime Planning Rules"));
    assert!(DOC.contains("`planning`"));
    assert!(DOC.contains("--expected-state-hash"));
    assert!(DOC.contains("--proposal <id|sender-did|nonce|state-hash>"));
    assert!(DOC.contains("Duplicate candidate IDs and stale state hashes are rejected"));
}

#[test]
fn doc_contains_runtime_recovery_check_rules() {
    assert!(DOC.contains("## Recovery Check Rules"));
    assert!(DOC.contains("`recovery-check`"));
    assert!(DOC.contains("--expected-state-version"));
    assert!(DOC.contains("--rejoin-attempt <node-id|state-version|state-hash|resume-token>"));
    assert!(DOC.contains("Replay resume tokens and version/hash mismatch scenarios are rejected"));
}

#[test]
fn doc_contains_runtime_mode_command_examples() {
    assert!(DOC.contains("`kamn-node --role processor --runtime-mode planning`"));
    assert!(DOC.contains("`kamn-node --role processor --runtime-mode recovery-check`"));
    assert!(DOC.contains("`kamn-node --role processor --runtime-mode daemon`"));
}

#[test]
fn doc_contains_runtime_daemon_rules() {
    assert!(DOC.contains("## Daemon Runtime Rules"));
    assert!(DOC.contains("--daemon-max-ticks"));
    assert!(DOC.contains("--daemon-tick-interval-ms"));
    assert!(DOC.contains("--daemon-lifecycle-event"));
    assert!(DOC.contains("tick-budget-exhausted"));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-node"));
    assert!(DOC.contains("cargo clippy -p kamn-node -- -D warnings"));
}

#[test]
fn doc_contains_docs_fast_lane_command_checks() {
    assert!(DOC.contains("cargo test -p kamn-node --test node_runtime_cli_docs"));
    assert!(DOC.contains("cargo test -p kamn-core --test runtime_network_docs"));
}

#[test]
fn regression_requires_invalid_output_mode_rule() {
    // Regression: #307
    assert!(DOC.contains("Invalid modes are rejected with explicit typed error."));
}

#[test]
fn regression_requires_invalid_profile_rule() {
    // Regression: #310
    assert!(DOC.contains("Invalid profiles are rejected with explicit typed error."));
}

#[test]
fn regression_requires_invalid_diagnostics_mode_rule() {
    // Regression: #313
    assert!(DOC.contains("Invalid diagnostics modes are rejected with explicit typed error."));
}

#[test]
fn regression_requires_runtime_planning_candidate_rules() {
    // Regression: #335
    assert!(
        DOC.contains("duplicate/stale runtime planning candidate rejection (`Regression: #335`)")
    );
}

#[test]
fn regression_requires_runtime_recovery_rejection_rules() {
    // Regression: #336
    assert!(DOC.contains("replay/version/hash recovery-check rejection (`Regression: #336`)"));
}

#[test]
fn regression_requires_runtime_recovery_error_rule_references() {
    // Regression: #337
    assert!(DOC.contains("ConfigError::InvalidExpectedStateVersion"));
    assert!(DOC.contains("ConfigError::InvalidRejoinAttemptArgument"));
    assert!(DOC.contains("ConfigError::RuntimeRecovery"));
}

#[test]
fn regression_requires_runtime_daemon_control_rules() {
    // Regression: #348
    assert!(DOC.contains("zero/invalid daemon bounded-loop control rejection (`Regression: #348`)"));
}

#[test]
fn regression_requires_runtime_daemon_lifecycle_rules() {
    // Regression: #349
    assert!(DOC.contains("invalid daemon lifecycle transition rejection (`Regression: #349`)"));
}
