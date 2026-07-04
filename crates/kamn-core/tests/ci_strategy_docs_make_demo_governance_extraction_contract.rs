use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "tests/ci_strategy_docs.rs";
const ROOT_CAP: usize = 520;
const MODULE_ROOT: &str = "tests/ci_strategy_docs/make_demo_governance_contract_tests.rs";
const MODULE_FILES: &[&str] = &[
    "tests/ci_strategy_docs/make_demo_governance_contract_tests/workflow_contract_tests.rs",
    "tests/ci_strategy_docs/make_demo_governance_contract_tests/wrapper_family_budget_contract_tests.rs",
    "tests/ci_strategy_docs/make_demo_governance_contract_tests/local_kolme_lane_contract_tests.rs",
    "tests/ci_strategy_docs/make_demo_governance_contract_tests/fast_gate_threshold_contract_tests.rs",
    "tests/ci_strategy_docs/make_demo_governance_contract_tests/support.rs",
];

fn line_count(path: &str) -> usize {
    fs::read_to_string(repo_path(path))
        .expect("extracted ci strategy docs fixture should be readable")
        .lines()
        .count()
}

fn read(path: &str) -> String {
    fs::read_to_string(repo_path(path)).unwrap_or_default()
}

fn repo_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

#[test]
fn ci_strategy_docs_make_demo_governance_extraction_contract() {
    let root = read(ROOT);
    assert_root_markers(&root);
    assert_root_budget();
    assert_module_root_exists();
    assert_extracted_files_fit_budget();
}

fn assert_root_markers(root: &str) {
    assert!(
        root.contains("#[path = \"ci_strategy_docs/make_demo_governance_contract_tests.rs\"]"),
        "root missing make/demo governance module path marker"
    );
    assert!(
        root.contains("mod make_demo_governance_contract_tests;"),
        "root missing make/demo governance module declaration"
    );
    assert!(
        !root.contains("fn doc_contains_make_and_demo_scope_contract_rules()"),
        "root still contains inline make/demo governance contract"
    );
}

fn assert_root_budget() {
    let lines = line_count(ROOT);
    assert!(
        lines <= ROOT_CAP,
        "root line count {lines} exceeds staged cap {ROOT_CAP}"
    );
}

fn assert_module_root_exists() {
    let path = repo_path(MODULE_ROOT);
    let path_display = path.display();
    assert!(path.exists(), "missing module root {path_display}");
}

fn assert_extracted_files_fit_budget() {
    for path in MODULE_FILES {
        let full_path = repo_path(path);
        let full_path_display = full_path.display();
        assert!(
            full_path.exists(),
            "missing extracted file {full_path_display}"
        );
        let lines = line_count(path);
        assert!(
            lines <= 200,
            "extracted file {path} exceeds 200 lines with {lines}"
        );
    }
}
