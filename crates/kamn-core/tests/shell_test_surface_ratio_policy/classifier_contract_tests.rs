use crate::support::is_docs_governance_rust_test_file;
use std::path::Path;

#[test]
fn unit_docs_governance_rust_test_classifier_detects_docs_contract_patterns() {
    assert!(is_docs_governance_rust_test_file(Path::new(
        "crates/kamn-core/tests/runtime_network_docs.rs"
    )));
    assert!(is_docs_governance_rust_test_file(Path::new(
        "crates/kamn-core/tests/review_r53_docs_contract.rs"
    )));
    assert!(is_docs_governance_rust_test_file(Path::new(
        "crates/kamn-core/tests/missing_docs_policy.rs"
    )));
    assert!(!is_docs_governance_rust_test_file(Path::new(
        "crates/kamn-core/tests/task_operations.rs"
    )));
}
