const DOC: &str = include_str!("../../../docs/foundation/reputation-state-model.md");

#[test]
fn doc_contains_prd_metrics_and_persistence_contract() {
    assert!(DOC.contains("## PRD 8.1 Metrics Coverage"));
    assert!(DOC.contains("trust_score"));
    assert!(DOC.contains("delivery_rate"));
    assert!(DOC.contains("endorsements"));
    assert!(DOC.contains("verified_capabilities"));
    assert!(DOC.contains("## Persistence Contract"));
    assert!(DOC.contains("kamn.reputation.scores:agent:<method-specific-id>"));
}

#[test]
fn doc_contains_error_handling_and_validation_rules() {
    assert!(DOC.contains("## Validation and Error Handling"));
    assert!(DOC.contains("Invalid agent DID"));
    assert!(DOC.contains("Duplicate endorsement IDs"));
    assert!(DOC.contains("Trust score updates reject values above 1000"));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --test reputation_state_model"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_upper_bound_score_inclusive_rule() {
    // Regression: #215
    assert!(DOC.contains("Trust score boundary checks are inclusive for `1000`."));
}
