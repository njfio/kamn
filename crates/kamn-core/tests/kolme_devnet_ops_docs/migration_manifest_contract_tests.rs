use super::docs_assert_support::{assert_plan_contains_all};

const PLAN_CONTAINS_LANE_MIGRATION_MATRIX_CONTRACT_PLAN_MARKERS: &[&str] = &[
    "## Lane Migration Matrix (Issue #1721)",
    "fixtures/kolme_compatibility/lane_migration_matrix.json",
    "kamn.kolme.lane-migration-matrix.v1",
    "check_lane_migration_matrix_policy.py",
    "test_check_lane_migration_matrix_policy.sh",
    "kolme.local.fork.rust_matrix",
];

#[test]
fn plan_contains_lane_migration_matrix_contract() {
    assert_plan_contains_all(PLAN_CONTAINS_LANE_MIGRATION_MATRIX_CONTRACT_PLAN_MARKERS, "plan_contains_lane_migration_matrix_contract");
}

const PLAN_CONTAINS_TRANCHE1_MANIFEST_MIGRATION_CONTRACT_PLAN_MARKERS: &[&str] = &[
    "## Tranche-1 Manifest Migration (Issue #1722)",
    "scripts/ci/test_kolme_tranche1_manifest_migration_contract.sh",
    "scripts/framework/manifests/kolme_snapshot_drift_contract_lane.json",
    "scripts/framework/manifests/kolme_notifications_consumer_contract_lane.json",
    "scripts/framework/manifests/kolme_block_fallback_reconciliation_contract_lane.json",
    "scripts/kolme/contracts/block_fallback_reconciliation_contract_lane.py",
    "Combined wrapper shell LOC for the tranche remains `<= 60`.",
];

#[test]
fn plan_contains_tranche1_manifest_migration_contract() {
    assert_plan_contains_all(PLAN_CONTAINS_TRANCHE1_MANIFEST_MIGRATION_CONTRACT_PLAN_MARKERS, "plan_contains_tranche1_manifest_migration_contract");
}

const PLAN_CONTAINS_RUNTIME_NONCE_MANIFEST_MIGRATION_CONTRACT_PLAN_MARKERS: &[&str] = &[
    "## Runtime+Nonce Manifest Migration (Issue #1763)",
    "scripts/ci/test_kolme_runtime_nonce_manifest_migration_contract.sh",
    "scripts/framework/manifests/kolme_runtime_commit_adapter_contract_lane.json",
    "scripts/framework/manifests/kolme_runtime_commit_replay_contract_lane.json",
    "scripts/framework/manifests/kolme_nonce_broadcast_parity_contract_lane.json",
    "scripts/kolme/contracts/runtime_commit_replay_contract_lane.py",
    "Combined wrapper shell LOC for this runtime/nonce tranche remains `<= 120`.",
];

#[test]
fn plan_contains_runtime_nonce_manifest_migration_contract() {
    assert_plan_contains_all(PLAN_CONTAINS_RUNTIME_NONCE_MANIFEST_MIGRATION_CONTRACT_PLAN_MARKERS, "plan_contains_runtime_nonce_manifest_migration_contract");
}

const PLAN_CONTAINS_VERSION_MATRIX_MANIFEST_MIGRATION_CONTRACT_PLAN_MARKERS: &[&str] = &[
    "## Version+Matrix Manifest Migration (Issue #1765)",
    "scripts/ci/test_kolme_version_matrix_manifest_migration_contract.sh",
    "scripts/framework/manifests/kolme_version_compatibility_contract_lane.json",
    "scripts/framework/manifests/kolme_local_fork_rust_test_matrix_contract_lane.json",
    "scripts/framework/manifests/kolme_local_heavy_validation_matrix_contract_lane.json",
    "scripts/kolme/contracts/local_heavy_validation_matrix_contract_lane.py",
    "Combined wrapper shell LOC for this version/matrix tranche remains `<= 120`.",
];

#[test]
fn plan_contains_version_matrix_manifest_migration_contract() {
    assert_plan_contains_all(PLAN_CONTAINS_VERSION_MATRIX_MANIFEST_MIGRATION_CONTRACT_PLAN_MARKERS, "plan_contains_version_matrix_manifest_migration_contract");
}

const PLAN_CONTAINS_PROFILE_SELFTEST_PORTABILITY_MANIFEST_MIGRATION_CONTRACT_PLAN_MARKERS: &[&str] = &[
    "## Profile+SelfTest+Portability Manifest Migration (Issue #1767)",
    "scripts/ci/test_kolme_profile_selftest_portability_manifest_migration_contract.sh",
    "scripts/framework/manifests/kolme_local_fork_profile_preflight_contract_lane.json",
    "scripts/framework/manifests/kolme_local_fork_self_test_contract_lane.json",
    "scripts/framework/manifests/kolme_local_fork_portability_preflight_contract_lane.json",
    "scripts/kolme/contracts/local_fork_self_test_contract_lane.py",
    "Combined wrapper shell LOC for this profile/self-test/portability tranche remains `<= 120`.",
];

#[test]
fn plan_contains_profile_selftest_portability_manifest_migration_contract() {
    assert_plan_contains_all(PLAN_CONTAINS_PROFILE_SELFTEST_PORTABILITY_MANIFEST_MIGRATION_CONTRACT_PLAN_MARKERS, "plan_contains_profile_selftest_portability_manifest_migration_contract");
}

const PLAN_CONTAINS_RUNTIME_TRIADIC_BOOTSTRAP_E2E_MANIFEST_MIGRATION_CONTRACT_PLAN_MARKERS: &[&str] = &[
    "## Runtime+Triadic+Bootstrap+E2E Manifest Migration (Issue #1769)",
    "scripts/ci/test_kolme_runtime_triadic_bootstrap_e2e_manifest_migration_contract.sh",
    "scripts/framework/manifests/kolme_runtime_commit_contract_lane.json",
    "scripts/framework/manifests/kolme_triadic_devnet_smoke_contract_lane.json",
    "scripts/framework/manifests/kolme_local_bootstrap_health_checks_contract_lane.json",
    "scripts/framework/manifests/kolme_local_e2e_integration_contract_lane.json",
    "scripts/kolme/contracts/local_e2e_integration_contract_lane.py",
    "Combined wrapper shell LOC for this runtime/triadic/bootstrap/e2e tranche remains `<= 160`.",
];

#[test]
fn plan_contains_runtime_triadic_bootstrap_e2e_manifest_migration_contract() {
    assert_plan_contains_all(PLAN_CONTAINS_RUNTIME_TRIADIC_BOOTSTRAP_E2E_MANIFEST_MIGRATION_CONTRACT_PLAN_MARKERS, "plan_contains_runtime_triadic_bootstrap_e2e_manifest_migration_contract");
}

const PLAN_CONTAINS_BOOTSTRAP_CONFORMANCE_RUNTIME_PROCESS_MANIFEST_MIGRATION_CONTRACT_PLAN_MARKERS: &[&str] = &[
    "## Bootstrap+Conformance+Runtime+Process Manifest Migration (Issue #1771)",
    "scripts/ci/test_kolme_bootstrap_conformance_runtime_process_manifest_migration_contract.sh",
    "scripts/framework/manifests/kolme_local_kolme_fork_bootstrap_readiness_contract_lane.json",
    "scripts/framework/manifests/kolme_local_kolme_live_api_conformance_contract_lane.json",
    "scripts/framework/manifests/kolme_local_kamn_live_runtime_integration_contract_lane.json",
    "scripts/framework/manifests/kolme_local_kolme_fork_process_lifecycle_contract_lane.json",
    "scripts/kolme/contracts/local_kolme_fork_process_lifecycle_contract_lane.py",
    "Combined wrapper shell LOC for this bootstrap/conformance/runtime/process tranche remains `<= 160`.",
];

#[test]
fn plan_contains_bootstrap_conformance_runtime_process_manifest_migration_contract() {
    assert_plan_contains_all(PLAN_CONTAINS_BOOTSTRAP_CONFORMANCE_RUNTIME_PROCESS_MANIFEST_MIGRATION_CONTRACT_PLAN_MARKERS, "plan_contains_bootstrap_conformance_runtime_process_manifest_migration_contract");
}

const PLAN_CONTAINS_PARITY_DEMO_REAL_PROCESS_MANIFEST_MIGRATION_CONTRACT_PLAN_MARKERS: &[&str] = &[
    "## Parity+Demo+Real-Process Manifest Migration (Issue #1773)",
    "scripts/ci/test_kolme_parity_demo_real_process_manifest_migration_contract.sh",
    "scripts/framework/manifests/kolme_fast_gate_native_api_parity_contract_lane.json",
    "scripts/framework/manifests/kolme_local_native_api_parity_live_proof_contract_lane.json",
    "scripts/framework/manifests/kolme_local_signed_to_kolme_demo_contract_lane.json",
    "scripts/framework/manifests/kolme_local_kolme_fork_checkout_bootstrap_contract_lane.json",
    "scripts/framework/manifests/kolme_local_kolme_fork_real_process_contract_lane.json",
    "scripts/kolme/contracts/local_kolme_fork_real_process_contract_lane.py",
    "Combined wrapper shell LOC for this parity/demo/real-process tranche remains `<= 200`.",
];

#[test]
fn plan_contains_parity_demo_real_process_manifest_migration_contract() {
    assert_plan_contains_all(PLAN_CONTAINS_PARITY_DEMO_REAL_PROCESS_MANIFEST_MIGRATION_CONTRACT_PLAN_MARKERS, "plan_contains_parity_demo_real_process_manifest_migration_contract");
}
