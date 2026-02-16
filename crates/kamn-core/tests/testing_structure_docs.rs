const DOC: &str = include_str!("../../../docs/testing/structure.md");

#[test]
fn doc_contains_main_tests_decomposition_drift_and_budget_markers() {
    assert!(DOC.contains("## Main Tests Decomposition Drift Cases (Issue #4452)"));
    assert!(DOC.contains(
        "main_tests_decomposition_reason_taxonomy_version=kamn.testing.main-tests-decomposition-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "main_tests_decomposition_reason_codes_csv=main_tests_domain_module_missing,main_tests_inline_monolith_reintroduced,main_tests_structural_budget_boundary_exceeded"
    ));
    assert!(DOC.contains("main_tests_decomposition_status=verified"));
    assert!(DOC.contains("main_tests_structural_budget_status=verified"));
}

#[test]
fn doc_contains_main_tests_decomposition_and_budget_guard_commands() {
    assert!(DOC
        .contains("cargo test -p kamn-node --test main_module_extraction_contract -- --nocapture"));
    assert!(DOC.contains("cargo test -p kamn-core --test testing_structure_docs -- --nocapture"));
    assert!(DOC.contains("bash scripts/ci/test_check_test_harness_loc_soft_budget.sh"));
    assert!(DOC.contains("bash scripts/ci/test_run_test_harness_loc_soft_budget_contract_lane.sh"));
}
