use std::fs;
use std::path::Path;

const ROOT: &str = "crates/kamn-core/tests/ci_strategy_docs.rs";
const ROOT_CAP: usize = 520;
const MODULE_ROOT: &str =
    "crates/kamn-core/tests/ci_strategy_docs/make_demo_governance_contract_tests.rs";
const MODULE_FILES: &[&str] = &[
    "crates/kamn-core/tests/ci_strategy_docs/make_demo_governance_contract_tests/workflow_contract_tests.rs",
    "crates/kamn-core/tests/ci_strategy_docs/make_demo_governance_contract_tests/wrapper_family_budget_contract_tests.rs",
    "crates/kamn-core/tests/ci_strategy_docs/make_demo_governance_contract_tests/local_kolme_lane_contract_tests.rs",
    "crates/kamn-core/tests/ci_strategy_docs/make_demo_governance_contract_tests/fast_gate_threshold_contract_tests.rs",
    "crates/kamn-core/tests/ci_strategy_docs/make_demo_governance_contract_tests/support.rs",
];

fn line_count(path: &str) -> usize {
    fs::read_to_string(path).unwrap().lines().count()
}

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

#[test]
fn ci_strategy_docs_make_demo_governance_extraction_contract() {
    let root = read(ROOT);
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
    assert!(
        line_count(ROOT) <= ROOT_CAP,
        "root line count {} exceeds staged cap {}",
        line_count(ROOT),
        ROOT_CAP
    );

    assert!(Path::new(MODULE_ROOT).exists(), "missing module root {MODULE_ROOT}");
    for path in MODULE_FILES {
        assert!(Path::new(path).exists(), "missing extracted file {path}");
        assert!(
            line_count(path) <= 200,
            "extracted file {path} exceeds 200 lines with {}",
            line_count(path)
        );
    }
}
