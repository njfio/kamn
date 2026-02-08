const DOC: &str = include_str!("../../../docs/foundation/reputation-signal-routing.md");

#[test]
fn doc_contains_signal_integration_model() {
    assert!(DOC.contains("## Signal Integration Model"));
    assert!(DOC.contains("endorsements"));
    assert!(DOC.contains("disputes"));
    assert!(DOC.contains("verified capabilities"));
    assert!(DOC.contains("rank_agents_for_routing"));
    assert!(DOC.contains("rank_listings_by_reputation"));
}

#[test]
fn doc_contains_error_handling_and_tiebreak_rules() {
    assert!(DOC.contains("## Validation and Error Handling"));
    assert!(DOC.contains("Invalid candidate DID"));
    assert!(DOC.contains("Missing reputation records"));
    assert!(DOC.contains("deterministic tie-break uses agent DID lexical order"));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --test reputation_signal_routing"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_did_tiebreak_rule() {
    // Regression: #211
    assert!(DOC.contains("Tie scores are resolved by DID lexical order."));
}
