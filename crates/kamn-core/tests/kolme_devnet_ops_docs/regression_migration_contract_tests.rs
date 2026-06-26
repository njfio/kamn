use super::docs_assert_support::assert_plan_contains_all;

const REGRESSION_REQUIRES_FAILOVER_SYNC_BUDGET_AND_SCHEDULED_CADENCE_GUARDS_PLAN_MARKERS:
    &[&str] = &["Failover/sync budget overruns and unscheduled deep-lane execution fail closed"];

#[test]
fn regression_requires_failover_sync_budget_and_scheduled_cadence_guards() {
    assert_plan_contains_all(
        REGRESSION_REQUIRES_FAILOVER_SYNC_BUDGET_AND_SCHEDULED_CADENCE_GUARDS_PLAN_MARKERS,
        "regression_requires_failover_sync_budget_and_scheduled_cadence_guards",
    );
}

const REGRESSION_REQUIRES_RUNTIME_COMMIT_ADAPTER_REASON_CODE_GUARD_PLAN_MARKERS: &[&str] = &[
    "runtime commit adapter replay/finality reason-code drift fails closed (`Regression: #980`).",
];

#[test]
fn regression_requires_runtime_commit_adapter_reason_code_guard() {
    assert_plan_contains_all(
        REGRESSION_REQUIRES_RUNTIME_COMMIT_ADAPTER_REASON_CODE_GUARD_PLAN_MARKERS,
        "regression_requires_runtime_commit_adapter_reason_code_guard",
    );
}

const REGRESSION_REQUIRES_LOCAL_ONLY_HEAVY_MATRIX_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local-only heavy validation matrix requires explicit opt-in and remains excluded from PR fast-gate workflows (`Regression: #1405`).",
];

#[test]
fn regression_requires_local_only_heavy_matrix_guard_marker() {
    assert_plan_contains_all(
        REGRESSION_REQUIRES_LOCAL_ONLY_HEAVY_MATRIX_GUARD_MARKER_PLAN_MARKERS,
        "regression_requires_local_only_heavy_matrix_guard_marker",
    );
}

const REGRESSION_REQUIRES_LOCAL_ONLY_HEAVY_MATRIX_POLICY_CONTRACT_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local-only heavy validation matrix summary policy and contract-lane command/report drift remain fail-closed (`Regression: #1687`).",
];

#[test]
fn regression_requires_local_only_heavy_matrix_policy_contract_guard_marker() {
    assert_plan_contains_all(
        REGRESSION_REQUIRES_LOCAL_ONLY_HEAVY_MATRIX_POLICY_CONTRACT_GUARD_MARKER_PLAN_MARKERS,
        "regression_requires_local_only_heavy_matrix_policy_contract_guard_marker",
    );
}

const REGRESSION_REQUIRES_LOCAL_BOOTSTRAP_POLICY_CONTRACT_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local bootstrap health summary policy and contract-lane command/report drift remain fail-closed (`Regression: #1692`).",
];

#[test]
fn regression_requires_local_bootstrap_policy_contract_guard_marker() {
    assert_plan_contains_all(
        REGRESSION_REQUIRES_LOCAL_BOOTSTRAP_POLICY_CONTRACT_GUARD_MARKER_PLAN_MARKERS,
        "regression_requires_local_bootstrap_policy_contract_guard_marker",
    );
}

const REGRESSION_REQUIRES_LANE_MIGRATION_MATRIX_POLICY_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "lane migration matrix schema/order/required-lane drift remains fail-closed (`Regression: #1721`).",
];

#[test]
fn regression_requires_lane_migration_matrix_policy_guard_marker() {
    assert_plan_contains_all(
        REGRESSION_REQUIRES_LANE_MIGRATION_MATRIX_POLICY_GUARD_MARKER_PLAN_MARKERS,
        "regression_requires_lane_migration_matrix_policy_guard_marker",
    );
}

const REGRESSION_REQUIRES_TRANCHE1_MANIFEST_MIGRATION_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "tranche-1 manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1722`).",
];

#[test]
fn regression_requires_tranche1_manifest_migration_guard_marker() {
    assert_plan_contains_all(
        REGRESSION_REQUIRES_TRANCHE1_MANIFEST_MIGRATION_GUARD_MARKER_PLAN_MARKERS,
        "regression_requires_tranche1_manifest_migration_guard_marker",
    );
}

const REGRESSION_REQUIRES_RUNTIME_NONCE_MANIFEST_MIGRATION_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "runtime+nonce manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1763`).",
];

#[test]
fn regression_requires_runtime_nonce_manifest_migration_guard_marker() {
    assert_plan_contains_all(
        REGRESSION_REQUIRES_RUNTIME_NONCE_MANIFEST_MIGRATION_GUARD_MARKER_PLAN_MARKERS,
        "regression_requires_runtime_nonce_manifest_migration_guard_marker",
    );
}

const REGRESSION_REQUIRES_VERSION_MATRIX_MANIFEST_MIGRATION_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "version+matrix manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1765`).",
];

#[test]
fn regression_requires_version_matrix_manifest_migration_guard_marker() {
    assert_plan_contains_all(
        REGRESSION_REQUIRES_VERSION_MATRIX_MANIFEST_MIGRATION_GUARD_MARKER_PLAN_MARKERS,
        "regression_requires_version_matrix_manifest_migration_guard_marker",
    );
}

const REGRESSION_REQUIRES_PROFILE_SELFTEST_PORTABILITY_MANIFEST_MIGRATION_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "profile+self-test+portability manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1767`).",
];

#[test]
fn regression_requires_profile_selftest_portability_manifest_migration_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_PROFILE_SELFTEST_PORTABILITY_MANIFEST_MIGRATION_GUARD_MARKER_PLAN_MARKERS, "regression_requires_profile_selftest_portability_manifest_migration_guard_marker");
}

const REGRESSION_REQUIRES_RUNTIME_TRIADIC_BOOTSTRAP_E2E_MANIFEST_MIGRATION_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "runtime+triadic+bootstrap+e2e manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1769`).",
];

#[test]
fn regression_requires_runtime_triadic_bootstrap_e2e_manifest_migration_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_RUNTIME_TRIADIC_BOOTSTRAP_E2E_MANIFEST_MIGRATION_GUARD_MARKER_PLAN_MARKERS, "regression_requires_runtime_triadic_bootstrap_e2e_manifest_migration_guard_marker");
}

const REGRESSION_REQUIRES_BOOTSTRAP_CONFORMANCE_RUNTIME_PROCESS_MANIFEST_MIGRATION_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "bootstrap+conformance+runtime+process manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1771`).",
];

#[test]
fn regression_requires_bootstrap_conformance_runtime_process_manifest_migration_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_BOOTSTRAP_CONFORMANCE_RUNTIME_PROCESS_MANIFEST_MIGRATION_GUARD_MARKER_PLAN_MARKERS, "regression_requires_bootstrap_conformance_runtime_process_manifest_migration_guard_marker");
}

const REGRESSION_REQUIRES_PARITY_DEMO_REAL_PROCESS_MANIFEST_MIGRATION_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "parity+demo+real-process manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1773`).",
];

#[test]
fn regression_requires_parity_demo_real_process_manifest_migration_guard_marker() {
    assert_plan_contains_all(
        REGRESSION_REQUIRES_PARITY_DEMO_REAL_PROCESS_MANIFEST_MIGRATION_GUARD_MARKER_PLAN_MARKERS,
        "regression_requires_parity_demo_real_process_manifest_migration_guard_marker",
    );
}

const REGRESSION_REQUIRES_LOCAL_BOOTSTRAP_OPT_IN_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "deterministic bootstrap run mode fails closed without explicit local-only opt-in (`Regression: #1417`).",
];

#[test]
fn regression_requires_local_bootstrap_opt_in_guard_marker() {
    assert_plan_contains_all(
        REGRESSION_REQUIRES_LOCAL_BOOTSTRAP_OPT_IN_GUARD_MARKER_PLAN_MARKERS,
        "regression_requires_local_bootstrap_opt_in_guard_marker",
    );
}

const REGRESSION_REQUIRES_LOCAL_E2E_OPT_IN_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local-only heavy E2E lane run mode fails closed without explicit local-only opt-in (`Regression: #1418`).",
];

#[test]
fn regression_requires_local_e2e_opt_in_guard_marker() {
    assert_plan_contains_all(
        REGRESSION_REQUIRES_LOCAL_E2E_OPT_IN_GUARD_MARKER_PLAN_MARKERS,
        "regression_requires_local_e2e_opt_in_guard_marker",
    );
}

const REGRESSION_REQUIRES_LOCAL_E2E_POLICY_CONTRACT_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local-only heavy E2E lane summary policy and contract-lane decision/checkpoint drift remain fail-closed (`Regression: #1682`).",
];

#[test]
fn regression_requires_local_e2e_policy_contract_guard_marker() {
    assert_plan_contains_all(
        REGRESSION_REQUIRES_LOCAL_E2E_POLICY_CONTRACT_GUARD_MARKER_PLAN_MARKERS,
        "regression_requires_local_e2e_policy_contract_guard_marker",
    );
}

const REGRESSION_REQUIRES_SHARED_LOCAL_HEAVY_OPT_IN_HELPER_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "shared local-heavy opt-in helper wiring remains fail-closed across bootstrap/E2E/matrix lanes (`Regression: #1585`).",
];

#[test]
fn regression_requires_shared_local_heavy_opt_in_helper_guard_marker() {
    assert_plan_contains_all(
        REGRESSION_REQUIRES_SHARED_LOCAL_HEAVY_OPT_IN_HELPER_GUARD_MARKER_PLAN_MARKERS,
        "regression_requires_shared_local_heavy_opt_in_helper_guard_marker",
    );
}
