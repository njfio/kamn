use super::super::support::assert_checklist_contains_all;

const REGRESSION_REQUIRES_GOVERNANCE_SIMULATION_AND_VETO_GUARD_MARKER_MARKERS: &[&str] = &[
    "simulation/veto bypass attempts and tampered evidence bundles force `NO-GO` (`Regression: #733`).",
];

#[test]
fn regression_requires_governance_simulation_and_veto_guard_marker() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_GOVERNANCE_SIMULATION_AND_VETO_GUARD_MARKER_MARKERS, "regression_requires_governance_simulation_and_veto_guard_marker");
}

const REGRESSION_REQUIRES_GOVERNANCE_STAKE_SLASH_RISK_GUARD_MARKER_MARKERS: &[&str] = &[
    "unsafe threshold bypass attempts and tampered risk evidence force `NO-GO` (`Regression: #733`).",
];

#[test]
fn regression_requires_governance_stake_slash_risk_guard_marker() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_GOVERNANCE_STAKE_SLASH_RISK_GUARD_MARKER_MARKERS, "regression_requires_governance_stake_slash_risk_guard_marker");
}

const REGRESSION_REQUIRES_REPUTATION_DISPUTE_EVIDENCE_GUARD_MARKER_MARKERS: &[&str] = &[
    "tampered evidence hashes, score-adjustment limit bypasses, and closed-policy-window decisions force `NO-GO` (`Regression: #730`).",
];

#[test]
fn regression_requires_reputation_dispute_evidence_guard_marker() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_REPUTATION_DISPUTE_EVIDENCE_GUARD_MARKER_MARKERS, "regression_requires_reputation_dispute_evidence_guard_marker");
}

const REGRESSION_REQUIRES_TOKEN_LAUNCH_HANDOFF_EVIDENCE_GUARD_MARKER_MARKERS: &[&str] = &[
    "supply/allocation invariant drift and insufficient approvals force `NO-GO` (`Regression: #714`).",
];

#[test]
fn regression_requires_token_launch_handoff_evidence_guard_marker() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_TOKEN_LAUNCH_HANDOFF_EVIDENCE_GUARD_MARKER_MARKERS, "regression_requires_token_launch_handoff_evidence_guard_marker");
}

const REGRESSION_REQUIRES_TREASURY_DISBURSEMENT_APPROVAL_GUARD_MARKER_MARKERS: &[&str] = &[
    "insufficient approvals, approval-window closure, and daily-limit overruns force `NO-GO` (`Regression: #716`).",
];

#[test]
fn regression_requires_treasury_disbursement_approval_guard_marker() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_TREASURY_DISBURSEMENT_APPROVAL_GUARD_MARKER_MARKERS, "regression_requires_treasury_disbursement_approval_guard_marker");
}

const REGRESSION_REQUIRES_TREASURY_SHARED_CONTRACT_LANE_MARKER_MARKERS: &[&str] = &[
    "shared contract-lane module marker remains required for docs/contracts drift guard (`Regression: #1278`).",
];

#[test]
fn regression_requires_treasury_shared_contract_lane_marker() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_TREASURY_SHARED_CONTRACT_LANE_MARKER_MARKERS, "regression_requires_treasury_shared_contract_lane_marker");
}

const REGRESSION_REQUIRES_MAINNET_CUTOVER_DEPENDENCY_AND_APPROVAL_GUARDS_MARKERS: &[&str] = &[
    "unresolved/non-prior dependencies and insufficient approvals force `NO-GO` (`Regression: #705`).",
];

#[test]
fn regression_requires_mainnet_cutover_dependency_and_approval_guards() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_MAINNET_CUTOVER_DEPENDENCY_AND_APPROVAL_GUARDS_MARKERS, "regression_requires_mainnet_cutover_dependency_and_approval_guards");
}

const REGRESSION_REQUIRES_CUTOVER_ROLLBACK_EVIDENCE_GUARD_MARKER_MARKERS: &[&str] = &[
    "missing failed-checkpoint evidence and rollback-target hash mismatch force `NO-GO` (`Regression: #708`).",
];

#[test]
fn regression_requires_cutover_rollback_evidence_guard_marker() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_CUTOVER_ROLLBACK_EVIDENCE_GUARD_MARKER_MARKERS, "regression_requires_cutover_rollback_evidence_guard_marker");
}

const REGRESSION_REQUIRES_LAUNCH_CANARY_EVIDENCE_GUARD_MARKER_MARKERS: &[&str] = &[
    "missing probe evidence and failing critical-path probes force `NO-GO` (`Regression: #710`).",
];

#[test]
fn regression_requires_launch_canary_evidence_guard_marker() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_LAUNCH_CANARY_EVIDENCE_GUARD_MARKER_MARKERS, "regression_requires_launch_canary_evidence_guard_marker");
}

const REGRESSION_REQUIRES_LAUNCH_CANARY_SHARED_CONTRACT_LANE_MARKER_MARKERS: &[&str] = &[
    "shared contract-lane module marker remains required for docs/contracts drift guard (`Regression: #1286`).",
];

#[test]
fn regression_requires_launch_canary_shared_contract_lane_marker() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_LAUNCH_CANARY_SHARED_CONTRACT_LANE_MARKER_MARKERS, "regression_requires_launch_canary_shared_contract_lane_marker");
}

const REGRESSION_REQUIRES_POST_CUTOVER_SLO_EVIDENCE_GUARD_MARKER_MARKERS: &[&str] = &[
    "stale snapshots and incomplete SLO evidence force `NO-GO` (`Regression: #711`).",
];

#[test]
fn regression_requires_post_cutover_slo_evidence_guard_marker() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_POST_CUTOVER_SLO_EVIDENCE_GUARD_MARKER_MARKERS, "regression_requires_post_cutover_slo_evidence_guard_marker");
}

const REGRESSION_REQUIRES_POST_CUTOVER_SLO_SHARED_CONTRACT_LANE_MARKER_MARKERS: &[&str] = &[
    "shared contract-lane module marker remains required for docs/contracts drift guard (`Regression: #1282`).",
];

#[test]
fn regression_requires_post_cutover_slo_shared_contract_lane_marker() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_POST_CUTOVER_SLO_SHARED_CONTRACT_LANE_MARKER_MARKERS, "regression_requires_post_cutover_slo_shared_contract_lane_marker");
}

const REGRESSION_REQUIRES_SIGNER_INCIDENT_RECOVERY_STALE_ARTIFACT_GUARD_MARKER_MARKERS: &[&str] = &[
    "stale deep-lane artifacts, unscheduled deep-lane execution, and incident recovery policy drift force `NO-GO` (`Regression: #989`).",
];

#[test]
fn regression_requires_signer_incident_recovery_stale_artifact_guard_marker() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_SIGNER_INCIDENT_RECOVERY_STALE_ARTIFACT_GUARD_MARKER_MARKERS, "regression_requires_signer_incident_recovery_stale_artifact_guard_marker");
}
