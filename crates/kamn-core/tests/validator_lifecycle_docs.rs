const DOC: &str =
    include_str!("../../../docs/foundation/validator-lifecycle-quorum-reconfiguration.md");

#[test]
fn doc_contains_validator_lifecycle_scope_and_contracts() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("ValidatorLifecycleManager"));
    assert!(DOC.contains("ValidatorTransitionProof"));
    assert!(DOC.contains("ValidatorLifecycleError"));
}

#[test]
fn doc_contains_transition_proof_and_quorum_safety_rules() {
    assert!(DOC.contains("## Transition Proof and Validation Rules"));
    assert!(DOC.contains("## Quorum Safety and Rollback Rules"));
    assert!(DOC.contains("Offboarding is blocked when resulting validator count would fall below current quorum threshold."));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --test validator_lifecycle"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_quorum_breaking_offboard_block_rule() {
    // Regression: #195
    assert!(DOC.contains("Offboarding is blocked when resulting validator count would fall below current quorum threshold."));
}
