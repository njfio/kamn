use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "tests/kolme_devnet_ops_docs.rs";
const ROOT_CAP: usize = 120;
const MODULE_FILES: &[&str] = &[
    "tests/kolme_devnet_ops_docs/service_api_failover_contract_tests.rs",
    "tests/kolme_devnet_ops_docs/deploy_compat_contract_tests.rs",
    "tests/kolme_devnet_ops_docs/local_lane_contract_tests.rs",
    "tests/kolme_devnet_ops_docs/migration_manifest_contract_tests.rs",
    "tests/kolme_devnet_ops_docs/regression_migration_contract_tests.rs",
    "tests/kolme_devnet_ops_docs/regression_local_lane_contract_tests.rs",
    "tests/kolme_devnet_ops_docs/runtime_transport_contract_tests.rs",
];
const REQUIRED_MARKERS: &[&str] = &[
    "mod service_api_failover_contract_tests;",
    "mod deploy_compat_contract_tests;",
    "mod local_lane_contract_tests;",
    "mod migration_manifest_contract_tests;",
    "mod regression_migration_contract_tests;",
    "mod regression_local_lane_contract_tests;",
    "mod runtime_transport_contract_tests;",
];
const MOVED_TEST_MARKERS: &[&str] = &[
    "fn plan_contains_triadic_smoke_contract_commands()",
    "fn deploy_compat_contains_kolme_upgrade_compatibility_taxonomy_runbook_parity_markers()",
    "fn plan_contains_local_fork_sync_metadata_lane()",
    "fn plan_contains_lane_migration_matrix_contract()",
    "fn regression_requires_lane_migration_matrix_policy_guard_marker()",
    "fn regression_requires_local_fork_sync_metadata_guard_marker()",
    "fn plan_contains_runtime_transport_retry_reconnect_failure_taxonomy()",
];

#[test]
fn kolme_devnet_ops_docs_root_is_extracted() {
    let root = fs::read_to_string(repo_path(ROOT)).expect("read root");
    let lines = root.lines().count();
    assert!(
        lines <= ROOT_CAP,
        "expected {ROOT} <= {ROOT_CAP} lines after extraction, found {lines}"
    );
    for marker in REQUIRED_MARKERS {
        assert!(root.contains(marker), "missing root module marker: {marker}");
    }
    for marker in MOVED_TEST_MARKERS {
        assert!(!root.contains(marker), "moved test marker still present: {marker}");
    }
    for path in MODULE_FILES {
        let full = repo_path(path);
        assert!(full.exists(), "missing extracted module: {}", full.display());
        assert!(
            fs::read_to_string(&full).expect("read module").lines().count() <= 200,
            "extracted module exceeds 200 lines: {}",
            full.display()
        );
    }
}

fn repo_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}
