const DOC: &str = include_str!("../../../docs/foundation/invariants.md");

#[test]
fn doc_contains_runtime_invariant_harness_coverage_contract() {
    assert!(DOC.contains("## Runtime Invariant Harness Coverage (Issue #897)"));
    assert!(DOC.contains("run_lifecycle_property_contract_lane.sh"));
    assert!(DOC.contains("kamn.runtime.lifecycle-property-contract-report.v1"));
    assert!(DOC.contains("lifecycle_property_replay:v1"));
    assert!(DOC.contains("run_input_mutation_contract_lane.sh"));
    assert!(DOC.contains("kamn.runtime.input-mutation-contract-report.v1"));
    assert!(DOC.contains("input_mutation_replay:v1"));
    assert!(DOC.contains("run_concurrency_state_mutation_contract_lane.sh"));
    assert!(DOC.contains("run_invariant_fuzz_concurrency_contract_lane.sh"));
    assert!(DOC.contains("check_invariant_fuzz_concurrency_policy.sh"));
    assert!(DOC.contains("kamn.runtime.invariant-fuzz-concurrency-contract-report.v1"));
}

#[test]
fn regression_requires_dispute_refund_property_and_concurrency_contract_markers() {
    // Regression: #904
    assert!(DOC.contains("## Dispute/Refund Property and Concurrency Contracts (Issue #904)"));
    assert!(DOC.contains("dispute_refund_transition_contracts"));
    assert!(DOC.contains("run_lifecycle_property_contract_lane.sh"));
    assert!(DOC.contains("run_concurrency_state_mutation_contract_lane.sh"));
    assert!(DOC.contains("Regression: #904"));
}

#[test]
fn regression_requires_zk_witness_mutation_contract_markers() {
    // Regression: #994
    assert!(DOC.contains("run_zk_witness_mutation_contract_lane.sh"));
    assert!(DOC.contains("run_zk_witness_mutation_deep_lane.sh"));
    assert!(DOC.contains("KAMN_RUNTIME_ZK_WITNESS_MUTATION_DEEP"));
}
