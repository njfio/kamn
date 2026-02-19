use std::path::Path;

const ROOT_SUITE: &str = include_str!("task_escrow_proptest_invariants.rs");

#[test]
fn root_harness_declares_task_and_escrow_domain_modules() {
    assert!(
        ROOT_SUITE.contains("#[path = \"task_escrow_proptest_invariants/shared.rs\"]\nmod shared;")
    );
    assert!(ROOT_SUITE.contains(
        "#[path = \"task_escrow_proptest_invariants/task_domain.rs\"]\nmod task_domain;"
    ));
    assert!(ROOT_SUITE.contains(
        "#[path = \"task_escrow_proptest_invariants/escrow_domain.rs\"]\nmod escrow_domain;"
    ));
}

#[test]
fn modularized_suite_files_exist_and_are_tracked() {
    assert!(Path::new("tests/task_escrow_proptest_invariants/shared.rs").is_file());
    assert!(Path::new("tests/task_escrow_proptest_invariants/task_domain.rs").is_file());
    assert!(Path::new("tests/task_escrow_proptest_invariants/escrow_domain.rs").is_file());
}

#[test]
fn testing_strategy_doc_records_suite_modularization_conventions() {
    let doc = std::fs::read_to_string("../../docs/testing/strategy.md")
        .expect("testing strategy doc must exist for suite modularization policy");
    assert!(doc.contains("task_escrow_proptest_invariants"));
    assert!(doc.contains("domain modules"));
}
