const DOC: &str = include_str!("../../../docs/testing/invariant-and-fuzz-strategy.md");

#[test]
fn invariant_strategy_docs_pin_transition_proptest_suite_sources() {
    assert!(DOC.contains("crates/kamn-core/tests/task_escrow_proptest_invariants.rs"));
    assert!(DOC.contains("crates/kamn-core/tests/peer_lifecycle_proptest_invariants.rs"));
}

#[test]
fn invariant_strategy_docs_pin_transition_rejection_reason_codes() {
    assert!(DOC.contains("task_transition_invalid_edge"));
    assert!(DOC.contains("escrow_transition_invalid"));
    assert!(DOC.contains("runtime_peer_transition_invalid"));
}
