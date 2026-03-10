use crate::support::{assert_markers, DOC};

const FAST_VALIDATION_MARKERS: &[&str] = &[
    "## Fast and Cost-Effective Validation",
    "cargo test -p kamn-node",
    "cargo test -p kamn-core construct_lock",
    "cargo clippy -p kamn-node -- -D warnings",
];
const DOCS_FAST_LANE_MARKERS: &[&str] = &[
    "cargo test -p kamn-node --test node_runtime_cli_docs",
    "cargo test -p kamn-node --test architecture_navigation_docs",
    "cargo test -p kamn-core --test runtime_network_docs",
];
const DAEMON_FAST_LANE_MARKERS: &[&str] = &[
    "### Daemon-focused fast lane",
    "cargo test -p kamn-node integration_runtime_daemon_renders_bounded_completion_output",
    "cargo test -p kamn-node regression_runtime_daemon_rejects_invalid_lifecycle_transition",
    "cargo test -p kamn-node functional_runtime_daemon_applies_graceful_shutdown_signal",
    "cargo test -p kamn-node integration_runtime_daemon_shutdown_timeout_is_fail_closed",
    "cargo test -p kamn-node regression_runtime_kolme_live_rejects_provider_marker_drift",
];
const INVALID_OUTPUT_MODE_RULE: &[&str] = &["Invalid modes are rejected with explicit typed error."];
const INVALID_PROFILE_RULE: &[&str] = &["Invalid profiles are rejected with explicit typed error."];
const INVALID_DIAGNOSTICS_RULE: &[&str] = &["Invalid diagnostics modes are rejected with explicit typed error."];
const PLANNING_REGRESSION_RULE: &[&str] =
    &["duplicate/stale runtime planning candidate rejection (`Regression: #335`)"];
const RECOVERY_REJECTION_RULE: &[&str] =
    &["replay/version/hash recovery-check rejection (`Regression: #336`)"];
const RECOVERY_ERROR_RULES: &[&str] = &[
    "ConfigError::InvalidExpectedStateVersion",
    "ConfigError::InvalidRejoinAttemptArgument",
    "ConfigError::RuntimeRecovery",
];
const DAEMON_CONTROL_RULE: &[&str] =
    &["zero/invalid daemon bounded-loop control rejection (`Regression: #348`)"];
const DAEMON_LIFECYCLE_RULE: &[&str] =
    &["invalid daemon lifecycle transition rejection (`Regression: #349`)"];
const DAEMON_LEASE_GUARD_RULE: &[&str] =
    &["daemon lease guard no-lease/invalid-owner rejection (`Regression: #388`)"];
const KOLME_LIVE_GUARD_RULE: &[&str] =
    &["in-memory fallback and invalid signing profile rejection (`Regression: #2175`)"];
const KOLME_PROVIDER_DRIFT_RULE: &[&str] =
    &["provider marker drift rejection in live submit/finality flow (`Regression: #2176`)"];

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert_markers(DOC, FAST_VALIDATION_MARKERS, "node runtime CLI fast validation lane");
}

#[test]
fn doc_contains_docs_fast_lane_command_checks() {
    assert_markers(DOC, DOCS_FAST_LANE_MARKERS, "node runtime CLI docs fast lane");
}

#[test]
fn doc_contains_daemon_focused_fast_lane_commands() {
    assert_markers(DOC, DAEMON_FAST_LANE_MARKERS, "node runtime CLI daemon fast lane");
}

#[test]
fn regression_requires_invalid_output_mode_rule() {
    assert_markers(DOC, INVALID_OUTPUT_MODE_RULE, "node runtime CLI invalid output regression");
}

#[test]
fn regression_requires_invalid_profile_rule() {
    assert_markers(DOC, INVALID_PROFILE_RULE, "node runtime CLI invalid profile regression");
}

#[test]
fn regression_requires_invalid_diagnostics_mode_rule() {
    assert_markers(DOC, INVALID_DIAGNOSTICS_RULE, "node runtime CLI invalid diagnostics regression");
}

#[test]
fn regression_requires_runtime_planning_candidate_rules() {
    assert_markers(DOC, PLANNING_REGRESSION_RULE, "node runtime CLI planning regression");
}

#[test]
fn regression_requires_runtime_recovery_rejection_rules() {
    assert_markers(DOC, RECOVERY_REJECTION_RULE, "node runtime CLI recovery rejection regression");
}

#[test]
fn regression_requires_runtime_recovery_error_rule_references() {
    assert_markers(DOC, RECOVERY_ERROR_RULES, "node runtime CLI recovery error regressions");
}

#[test]
fn regression_requires_runtime_daemon_control_rules() {
    assert_markers(DOC, DAEMON_CONTROL_RULE, "node runtime CLI daemon control regression");
}

#[test]
fn regression_requires_runtime_daemon_lifecycle_rules() {
    assert_markers(DOC, DAEMON_LIFECYCLE_RULE, "node runtime CLI daemon lifecycle regression");
}

#[test]
fn regression_requires_runtime_daemon_lease_guard_rules() {
    assert_markers(DOC, DAEMON_LEASE_GUARD_RULE, "node runtime CLI daemon lease regression");
}

#[test]
fn regression_requires_runtime_kolme_live_guard_rules() {
    assert_markers(DOC, KOLME_LIVE_GUARD_RULE, "node runtime CLI kolme-live regression");
}

#[test]
fn regression_requires_runtime_kolme_live_provider_drift_guard_rules() {
    assert_markers(DOC, KOLME_PROVIDER_DRIFT_RULE, "node runtime CLI provider drift regression");
}
