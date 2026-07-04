use super::super::docs_assert_support::assert_plan_contains_all;

const PLAN_CONTAINS_STAGED_REHEARSAL_SIGNOFF_ARTIFACT_CONTRACT_PLAN_MARKERS: &[&str] = &[
    "## Staged Rehearsal Signoff Artifact Contract (Issue #3241)",
    "run_manifest_lane.sh --manifest scripts/framework/manifests/deploy_staging_rehearsal_contract_lane.json --phase contract",
    "check_staging_rehearsal_policy.sh",
    "kamn.release.staged-rehearsal-signoff.v1",
    "staged_rehearsal_signoff_status=verified|fail-closed",
];

#[test]
fn plan_contains_staged_rehearsal_signoff_artifact_contract() {
    assert_plan_contains_all(
        PLAN_CONTAINS_STAGED_REHEARSAL_SIGNOFF_ARTIFACT_CONTRACT_PLAN_MARKERS,
        "plan_contains_staged_rehearsal_signoff_artifact_contract",
    );
}

const PLAN_CONTAINS_FAST_GATE_NATIVE_API_PARITY_LANE_PLAN_MARKERS: &[&str] = &[
    "## Fast-Gate Native API Parity Contract Lane",
    "run_fast_gate_native_api_parity_contract_lane.sh",
    "check_fast_gate_native_api_parity_policy.py",
    "kamn.kolme.fast-gate-native-api-parity-summary.v1",
    "KAMN_KOLME_FAST_GATE_NATIVE_PARITY_MAX_SECONDS",
    "test_run_fast_gate_native_api_parity_contract_lane.sh",
];

#[test]
fn plan_contains_fast_gate_native_api_parity_lane() {
    assert_plan_contains_all(
        PLAN_CONTAINS_FAST_GATE_NATIVE_API_PARITY_LANE_PLAN_MARKERS,
        "plan_contains_fast_gate_native_api_parity_lane",
    );
}

const PLAN_CONTAINS_DETERMINISTIC_LOCAL_BOOTSTRAP_HEALTH_CHECKS_PLAN_MARKERS: &[&str] = &[
    "## Deterministic Local Bootstrap Health Checks",
    "run_local_bootstrap_health_checks.sh",
    "check_local_bootstrap_health_policy.py",
    "run_local_bootstrap_health_checks_contract_lane.sh",
    "kamn.kolme.local-bootstrap-summary.v1",
    "kamn.kolme.local-bootstrap-policy-report.v1",
    "KAMN_KOLME_LOCAL_HEAVY=1",
];

#[test]
fn plan_contains_deterministic_local_bootstrap_health_checks() {
    assert_plan_contains_all(
        PLAN_CONTAINS_DETERMINISTIC_LOCAL_BOOTSTRAP_HEALTH_CHECKS_PLAN_MARKERS,
        "plan_contains_deterministic_local_bootstrap_health_checks",
    );
}
