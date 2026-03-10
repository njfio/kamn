use std::fs;
use std::path::{Path, PathBuf};

const ROOT_MAX_LINES: usize = 200;
const LEAF_MAX_LINES: usize = 200;
const ROOT_FILE: &str = "crates/kamn-core/tests/service_api_ops_configuration_docs.rs";

#[test]
fn service_api_ops_configuration_docs_root_shell_budget_is_enforced() {
    assert!(line_count(repo_path(ROOT_FILE)) <= ROOT_MAX_LINES);
}

#[test]
fn service_api_ops_configuration_docs_root_declares_expected_modules() {
    let root = read(ROOT_FILE);
    for marker in expected_root_markers() {
        assert!(root.contains(marker), "missing root marker: {marker}");
    }
}

#[test]
fn service_api_ops_configuration_docs_root_no_longer_contains_representative_moved_tests() {
    let root = read(ROOT_FILE);
    for marker in moved_test_markers() {
        assert!(!root.contains(marker), "root still contains moved marker: {marker}");
    }
}

#[test]
fn service_api_ops_configuration_docs_extracted_files_exist_and_stay_bounded() {
    for rel in expected_leaf_files() {
        let path = repo_path(rel);
        assert!(path.is_file(), "missing extracted file: {rel}");
        assert!(line_count(path) <= LEAF_MAX_LINES, "leaf too large: {rel}");
    }
}

fn expected_root_markers() -> &'static [&'static str] {
    &[
        "mod dependency_supply_chain_contract_tests;",
        "mod compatibility_resilience_contract_tests;",
        "mod phase6_runtime_contract_tests;",
        "mod live_postgres_matrix_contract_tests;",
        "mod guardrail_signer_contract_tests;",
        "mod reconciliation_upgrade_contract_tests;",
    ]
}

fn moved_test_markers() -> &'static [&'static str] {
    &[
        "fn service_api_ops_configuration_contains_signer_secret_zeroization_controls()",
        "fn service_api_ops_configuration_contains_api_version_policy_markers()",
        "fn service_api_ops_configuration_contains_phase6_daemon_runtime_integration_markers()",
        "fn service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_mapping_markers(",
        "fn service_api_ops_configuration_contains_multi_signer_quorum_signature_decision_controls()",
        "fn service_api_ops_configuration_contains_upgrade_compatibility_marker_matrix_controls()",
    ]
}

fn expected_leaf_files() -> &'static [&'static str] {
    &[
        "crates/kamn-core/tests/service_api_ops_configuration_docs/dependency_supply_chain_contract_tests.rs",
        "crates/kamn-core/tests/service_api_ops_configuration_docs/compatibility_resilience_contract_tests.rs",
        "crates/kamn-core/tests/service_api_ops_configuration_docs/phase6_runtime_contract_tests.rs",
        "crates/kamn-core/tests/service_api_ops_configuration_docs/live_postgres_matrix_contract_tests.rs",
        "crates/kamn-core/tests/service_api_ops_configuration_docs/guardrail_signer_contract_tests.rs",
        "crates/kamn-core/tests/service_api_ops_configuration_docs/reconciliation_upgrade_contract_tests.rs",
    ]
}

fn repo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join(rel)
}

fn read(rel: &str) -> String {
    fs::read_to_string(repo_path(rel)).expect("contract fixture should be readable")
}

fn line_count(path: PathBuf) -> usize {
    fs::read_to_string(path)
        .expect("contract fixture should be readable")
        .lines()
        .count()
}
