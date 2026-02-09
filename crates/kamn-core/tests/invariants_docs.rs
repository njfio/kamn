const DOC: &str = include_str!("../../../docs/foundation/invariants.md");

#[test]
fn doc_contains_runtime_invariant_harness_coverage_contract() {
    assert!(DOC.contains("## Runtime Invariant Harness Coverage (Issue #897)"));
    assert!(DOC.contains("run_lifecycle_property_contract_lane.sh"));
    assert!(DOC.contains("run_input_mutation_contract_lane.sh"));
    assert!(DOC.contains("run_concurrency_state_mutation_contract_lane.sh"));
    assert!(DOC.contains("run_invariant_fuzz_concurrency_contract_lane.sh"));
    assert!(DOC.contains("check_invariant_fuzz_concurrency_policy.sh"));
    assert!(DOC.contains("kamn.runtime.invariant-fuzz-concurrency-contract-report.v1"));
}
