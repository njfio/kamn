use super::super::docs_assert_support::assert_plan_contains_all;

const PLAN_CONTAINS_LOCAL_ONLY_HEAVY_VALIDATION_MATRIX_PLAN_MARKERS: &[&str] = &[
    "## Local-Only Heavy Kolme Validation Matrix",
    "run_local_heavy_validation_matrix.sh",
    "check_local_heavy_validation_matrix_policy.py",
    "run_local_heavy_validation_matrix_contract_lane.sh",
    "--cargo-profile portable",
    "run_local_bootstrap_health_checks.sh",
    "run_version_compatibility_replay_deep_lane.sh",
    "kamn.kolme.local-heavy-validation-summary.v1",
    "kamn.kolme.local-heavy-validation-policy-report.v1",
];

#[test]
fn plan_contains_local_only_heavy_validation_matrix() {
    assert_plan_contains_all(
        PLAN_CONTAINS_LOCAL_ONLY_HEAVY_VALIDATION_MATRIX_PLAN_MARKERS,
        "plan_contains_local_only_heavy_validation_matrix",
    );
}

const PLAN_CONTAINS_LOCAL_ONLY_HEAVY_E2E_LANE_PLAN_MARKERS: &[&str] = &[
    "## Local-Only Heavy End-to-End Lane",
    "run_local_e2e_integration_lane.sh",
    "check_local_e2e_integration_policy.py",
    "run_local_e2e_integration_contract_lane.sh",
    "kamn.kolme.local-e2e-integration-summary.v1",
    "kamn.kolme.local-e2e-integration-policy-report.v1",
    "run_runtime_commit_adapter_contract_lane.sh",
    "run_live_transport_parity_contract_lane.sh",
];

#[test]
fn plan_contains_local_only_heavy_e2e_lane() {
    assert_plan_contains_all(
        PLAN_CONTAINS_LOCAL_ONLY_HEAVY_E2E_LANE_PLAN_MARKERS,
        "plan_contains_local_only_heavy_e2e_lane",
    );
}
