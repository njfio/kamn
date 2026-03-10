use std::fs;
use std::path::{Path, PathBuf};

const ROOT_FILE: &str = "src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_coherence_contract_tests.rs";
const ROOT_MAX_LINES: usize = 180;
const EXPECTED_ROOT_MARKERS: &[&str] = &[
    "include!(\"live_postgres_topology_contract_tests/topology_coherence_contract_tests/host_mode_cardinality_contract_tests.rs\");",
    "include!(\"live_postgres_topology_contract_tests/topology_coherence_contract_tests/host_pair_cardinality_contract_tests.rs\");",
    "include!(\"live_postgres_topology_contract_tests/topology_coherence_contract_tests/host_mode_host_pair_contract_tests.rs\");",
    "include!(\"live_postgres_topology_contract_tests/topology_coherence_contract_tests/lane_set_bundle_contract_tests.rs\");",
    "include!(\"live_postgres_topology_contract_tests/topology_coherence_contract_tests/fingerprint_hash_coherence_contract_tests.rs\");",
    "include!(\"live_postgres_topology_contract_tests/topology_coherence_contract_tests/hash_normalization_contract_tests.rs\");",
];
const EXPECTED_MODULE_FILES: &[&str] = &[
    "src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_coherence_contract_tests/host_mode_cardinality_contract_tests.rs",
    "src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_coherence_contract_tests/host_pair_cardinality_contract_tests.rs",
    "src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_coherence_contract_tests/host_mode_host_pair_contract_tests.rs",
    "src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_coherence_contract_tests/host_mode_host_pair_contract_tests/coherence_contract_tests.rs",
    "src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_coherence_contract_tests/host_mode_host_pair_contract_tests/cardinality_contract_tests.rs",
    "src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_coherence_contract_tests/lane_set_bundle_contract_tests.rs",
    "src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_coherence_contract_tests/lane_set_bundle_contract_tests/lane_count_contract_tests.rs",
    "src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_coherence_contract_tests/lane_set_bundle_contract_tests/lane_id_bundle_contract_tests.rs",
    "src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_coherence_contract_tests/fingerprint_hash_coherence_contract_tests.rs",
    "src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_coherence_contract_tests/fingerprint_hash_coherence_contract_tests/lane_fingerprint_bundle_contract_tests.rs",
    "src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_coherence_contract_tests/fingerprint_hash_coherence_contract_tests/lane_fingerprint_hash_contract_tests.rs",
    "src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_coherence_contract_tests/hash_normalization_contract_tests.rs",
    "src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_coherence_contract_tests/hash_normalization_contract_tests/order_normalization_contract_tests.rs",
    "src/main_tests/daemon_tests/live_postgres_topology_contract_tests/topology_coherence_contract_tests/hash_normalization_contract_tests/order_normalization_digest_contract_tests.rs",
];
const MOVED_TEST_MARKERS: &[&str] = &[
    "functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_cardinality_coherence_contract_is_canonical",
    "integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_pair_cardinality_coherence_is_stable",
    "functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_id_bundle_coherence_contract_is_canonical",
    "integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_coherence_is_stable",
    "integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_is_stable",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn read_repo_file(path: &str) -> String {
    let full_path = repo_root().join(path);
    fs::read_to_string(&full_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", full_path.display()))
}

#[test]
fn regression_topology_coherence_root_declares_extracted_submodules() {
    let root = read_repo_file(ROOT_FILE);
    for marker in EXPECTED_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "topology_coherence_contract_tests.rs missing extracted-module marker: {marker}"
        );
    }
}

#[test]
fn regression_topology_coherence_module_files_exist() {
    let root = repo_root();
    for path in EXPECTED_MODULE_FILES {
        let full_path = root.join(path);
        assert!(
            full_path.exists(),
            "expected topology coherence module file missing: {}",
            full_path.display()
        );
    }
}

#[test]
fn regression_topology_coherence_root_removes_moved_test_bodies() {
    let root = read_repo_file(ROOT_FILE);
    for marker in MOVED_TEST_MARKERS {
        assert!(
            !root.contains(marker),
            "topology_coherence_contract_tests.rs should not retain moved test marker: {marker}"
        );
    }
}

#[test]
fn regression_topology_coherence_root_stays_within_shell_budget() {
    let line_count = read_repo_file(ROOT_FILE).lines().count();
    assert!(
        line_count <= ROOT_MAX_LINES,
        "topology_coherence_contract_tests.rs should stay within {ROOT_MAX_LINES} lines after extraction, found {line_count}"
    );
}
