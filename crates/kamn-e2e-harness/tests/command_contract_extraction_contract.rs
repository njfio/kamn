use std::fs;
use std::path::{Path, PathBuf};

const ROOT_MAX_LINES: usize = 200;
const LEAF_MAX_LINES: usize = 200;
const ROOT_FILE: &str = "crates/kamn-e2e-harness/tests/command_contract.rs";

#[test]
fn command_contract_root_shell_budget_is_enforced() {
    assert!(line_count(repo_path(ROOT_FILE)) <= ROOT_MAX_LINES);
}

#[test]
fn command_contract_root_declares_expected_modules() {
    let root = read(ROOT_FILE);
    for marker in expected_root_markers() {
        assert!(root.contains(marker), "missing root marker: {marker}");
    }
}

#[test]
fn command_contract_root_no_longer_contains_moved_test_markers() {
    let root = read(ROOT_FILE);
    for marker in moved_test_markers() {
        assert!(!root.contains(marker), "root still contains moved marker: {marker}");
    }
}

#[test]
fn command_contract_extracted_files_exist_and_stay_bounded() {
    for rel in expected_leaf_files() {
        let path = repo_path(rel);
        assert!(path.is_file(), "missing extracted file: {rel}");
        assert!(line_count(path) <= LEAF_MAX_LINES, "leaf too large: {rel}");
    }
}

fn expected_root_markers() -> &'static [&'static str] {
    &[
        "mod parser_verify_contract_tests;",
        "mod phase_inventory_contract_tests;",
        "mod integration_runtime_contract_tests;",
        "mod external_execution_contract_tests;",
        "mod scenario_evidence_contract_tests;",
        "mod teardown_contract_tests;",
    ]
}

fn moved_test_markers() -> &'static [&'static str] {
    &[
        "fn spec_c01_parser_accepts_run_with_required_flags()",
        "fn spec_c08_phase_inventory_contains_prd_canonical_order()",
        "fn spec_c21_parser_accepts_sdk_direct_with_kolme_binary_only()",
        "fn spec_c47_parser_accepts_external_execution_flag()",
        "fn spec_c72_run_output_contains_scenario_results_in_selected_order()",
        "fn spec_c87_teardown_phase_is_pass_with_prd_step_inventory_in_sdk_mode()",
    ]
}

fn expected_leaf_files() -> &'static [&'static str] {
    &[
        "crates/kamn-e2e-harness/tests/command_contract/parser_verify_contract_tests.rs",
        "crates/kamn-e2e-harness/tests/command_contract/phase_inventory_contract_tests.rs",
        "crates/kamn-e2e-harness/tests/command_contract/integration_runtime_contract_tests.rs",
        "crates/kamn-e2e-harness/tests/command_contract/external_execution_contract_tests.rs",
        "crates/kamn-e2e-harness/tests/command_contract/scenario_evidence_contract_tests.rs",
        "crates/kamn-e2e-harness/tests/command_contract/teardown_contract_tests.rs",
    ]
}

fn repo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(rel)
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
