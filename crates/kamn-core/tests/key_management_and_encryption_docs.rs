const DOC: &str = include_str!("../../../docs/foundation/key-management-and-encryption.md");

#[test]
fn doc_contains_key_lifecycle_contract_scope() {
    assert!(DOC.contains("# Key Management and Encryption Contract Rules"));
    assert!(DOC.contains("run_key_hierarchy_invariant_contract_lane.sh"));
    assert!(DOC.contains("key_lifecycle_invariant_contract.py"));
    assert!(DOC.contains("kamn.key-lifecycle.invariant-evidence.v1"));
}

#[test]
fn regression_requires_replay_stale_activation_marker() {
    // Regression: #931
    assert!(DOC.contains("replay/stale key activation drift is rejected (`Regression: #931`)"));
    assert!(DOC.contains("check_key_lifecycle_invariant_policy.sh"));
}
