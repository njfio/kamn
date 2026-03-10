use super::super::support::assert_checklist_contains_all;

const REGRESSION_REQUIRES_FEDERATED_RUNTIME_TRUST_STORE_GUARD_MARKER_MARKERS: &[&str] = &[
    "runtime trust-store misses and quorum shortfalls must remain fail-closed with deterministic reason codes (`Regression: #1002`).",
];

#[test]
fn regression_requires_federated_runtime_trust_store_guard_marker() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_FEDERATED_RUNTIME_TRUST_STORE_GUARD_MARKER_MARKERS, "regression_requires_federated_runtime_trust_store_guard_marker");
}

const REGRESSION_REQUIRES_FEDERATED_DEEP_LANE_TAMPER_GUARD_MARKER_MARKERS: &[&str] = &[
    "stale/tampered federated handshake deep-lane summary artifacts must remain `NO-GO` (`Regression: #1003`).",
];

#[test]
fn regression_requires_federated_deep_lane_tamper_guard_marker() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_FEDERATED_DEEP_LANE_TAMPER_GUARD_MARKER_MARKERS, "regression_requires_federated_deep_lane_tamper_guard_marker");
}

const REGRESSION_REQUIRES_FEDERATED_DELEGATION_SETTLEMENT_EVIDENCE_GUARD_MARKER_MARKERS: &[&str] = &[
    "settlement reference drift, replay attempts, quorum shortfalls, and tampered final decisions force `NO-GO` (`Regression: #734`).",
];

#[test]
fn regression_requires_federated_delegation_settlement_evidence_guard_marker() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_FEDERATED_DELEGATION_SETTLEMENT_EVIDENCE_GUARD_MARKER_MARKERS, "regression_requires_federated_delegation_settlement_evidence_guard_marker");
}

const REGRESSION_REQUIRES_KOLME_INCOMPATIBLE_UPGRADE_SIGNATURE_GUARD_MARKER_MARKERS: &[&str] = &[
    "incompatible upgrade signature (`kamn 1.2.x` + `kolme 0.14.x`) remains blocked (`Regression: #775`).",
];

#[test]
fn regression_requires_kolme_incompatible_upgrade_signature_guard_marker() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_KOLME_INCOMPATIBLE_UPGRADE_SIGNATURE_GUARD_MARKER_MARKERS, "regression_requires_kolme_incompatible_upgrade_signature_guard_marker");
}

const REGRESSION_REQUIRES_KOLME_RUNTIME_COMMIT_REPLAY_GUARD_MARKER_MARKERS: &[&str] = &[
    "runtime commit replay/tamper mismatches and non-final receipts force `NO-GO` (`Regression: #827`).",
];

#[test]
fn regression_requires_kolme_runtime_commit_replay_guard_marker() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_KOLME_RUNTIME_COMMIT_REPLAY_GUARD_MARKER_MARKERS, "regression_requires_kolme_runtime_commit_replay_guard_marker");
}

const REGRESSION_REQUIRES_ADAPTER_RUNTIME_COMMIT_REPLAY_GUARD_MARKER_MARKERS: &[&str] = &[
    "adapter transport/provider mismatch and non-final receipt reason-code checks remain fail-closed (`Regression: #980`).",
];

#[test]
fn regression_requires_adapter_runtime_commit_replay_guard_marker() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_ADAPTER_RUNTIME_COMMIT_REPLAY_GUARD_MARKER_MARKERS, "regression_requires_adapter_runtime_commit_replay_guard_marker");
}

const REGRESSION_REQUIRES_KOLME_FORK_RELEASE_DRIFT_GUARD_MARKER_MARKERS: &[&str] = &[
    "fork release-tag drift remains blocked (`Regression: #1401`).",
];

#[test]
fn regression_requires_kolme_fork_release_drift_guard_marker() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_KOLME_FORK_RELEASE_DRIFT_GUARD_MARKER_MARKERS, "regression_requires_kolme_fork_release_drift_guard_marker");
}

const REGRESSION_REQUIRES_KOLME_FORK_POLICY_CHECKER_GUARD_MARKER_MARKERS: &[&str] = &[
    "fork policy checker rejects malformed schema, tuple mismatch, and missing required reason codes (`Regression: #1402`).",
];

#[test]
fn regression_requires_kolme_fork_policy_checker_guard_marker() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_KOLME_FORK_POLICY_CHECKER_GUARD_MARKER_MARKERS, "regression_requires_kolme_fork_policy_checker_guard_marker");
}
