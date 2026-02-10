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
fn doc_contains_dispute_evidence_contract() {
    assert!(DOC.contains("## Dispute Evidence Contract"));
    assert!(DOC.contains("generate_reputation_dispute_evidence_bundle.sh"));
    assert!(DOC.contains("check_reputation_dispute_policy.sh"));
    assert!(DOC.contains("reputation_dispute_contract.py"));
    assert!(DOC.contains("run_reputation_dispute_contract_lane.sh"));
    assert!(DOC.contains("run_reputation_dispute_deep_lane.sh"));
    assert!(DOC.contains("run_reputation_dispute_matrix.py"));
    assert!(DOC.contains("fixtures/reputation_dispute/replay_cases.json"));
}

#[test]
fn doc_contains_signal_quarantine_evidence_contract() {
    assert!(DOC.contains("## Signal Quarantine Evidence Contract"));
    assert!(DOC.contains("generate_reputation_signal_quarantine_evidence_bundle.sh"));
    assert!(DOC.contains("check_reputation_signal_quarantine_policy.sh"));
    assert!(DOC.contains("run_reputation_signal_quarantine_contract_lane.sh"));
    assert!(DOC.contains("reputation_signal_quarantine_contract.py"));
}

#[test]
fn regression_requires_did_tiebreak_rule() {
    // Regression: #211
    assert!(DOC.contains("Tie scores are resolved by DID lexical order."));
}

#[test]
fn regression_requires_signal_quarantine_guard_marker() {
    // Regression: #935
    assert!(DOC.contains(
        "tampered reason keys/reason codes and ingestion-action mismatches force `NO-GO` (`Regression: #935`)."
    ));
}

#[test]
fn regression_requires_reputation_dispute_evidence_guard_marker() {
    // Regression: #730
    assert!(DOC.contains(
        "tampered evidence hashes, score-adjustment limit bypasses, and closed-policy-window decisions force `NO-GO` (`Regression: #730`)."
    ));
}
