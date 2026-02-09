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
fn doc_contains_dispute_evidence_bundle_contract() {
    assert!(DOC.contains("## Deterministic Reputation Dispute Evidence Contract"));
    assert!(DOC.contains("generate_reputation_dispute_evidence_bundle.sh"));
    assert!(DOC.contains("check_reputation_dispute_policy.sh"));
    assert!(DOC.contains("reason_key"));
    assert!(DOC.contains("run_reputation_dispute_contract_lane.sh"));
    assert!(DOC.contains("run_reputation_dispute_deep_lane.sh"));
    assert!(DOC.contains("run_reputation_dispute_matrix.py"));
    assert!(DOC.contains("fixtures/reputation_dispute/replay_cases.json"));
}

#[test]
fn doc_contains_signal_quarantine_contract() {
    assert!(DOC.contains("## Deterministic Reputation Signal Quarantine Contract"));
    assert!(DOC.contains("generate_reputation_signal_quarantine_evidence_bundle.sh"));
    assert!(DOC.contains("check_reputation_signal_quarantine_policy.sh"));
    assert!(DOC.contains("run_reputation_signal_quarantine_contract_lane.sh"));
    assert!(DOC.contains("REPUTATION_QUARANTINE_MAX_SECONDS"));
    assert!(DOC.contains("reason_codes"));
}

#[test]
fn doc_contains_recovery_reversal_contract() {
    assert!(DOC.contains("## Deterministic Reputation Recovery Reversal Contract"));
    assert!(DOC.contains("generate_reputation_recovery_evidence_bundle.sh"));
    assert!(DOC.contains("check_reputation_recovery_policy.sh"));
    assert!(DOC.contains("run_reputation_recovery_contract_lane.sh"));
    assert!(DOC.contains("reputation_recovery_contract.py"));
    assert!(DOC.contains("REPUTATION_RECOVERY_MAX_SECONDS"));
    assert!(DOC.contains("recovery_action"));
}

#[test]
fn doc_contains_weighted_decay_and_antigaming_contract() {
    assert!(DOC.contains("## Weighted Decay and Anti-Gaming Threshold Contract"));
    assert!(DOC.contains("run_weighted_decay_contract_lane.sh"));
    assert!(DOC.contains("run_weighted_decay_deep_lane.sh"));
    assert!(DOC.contains("run_weighted_decay_matrix.py"));
    assert!(DOC.contains("fixtures/reputation_decay/compact_cases.json"));
    assert!(DOC.contains("fixtures/reputation_decay/adversarial_cases.json"));
}

#[test]
fn regression_requires_upper_bound_score_inclusive_rule() {
    // Regression: #215
    assert!(DOC.contains("Trust score boundary checks are inclusive for `1000`."));
}

#[test]
fn regression_requires_reputation_dispute_tamper_guard_marker() {
    // Regression: #730
    assert!(DOC.contains(
        "tampered evidence hashes, score-adjustment limit bypasses, and closed-policy-window decisions force `NO-GO` (`Regression: #730`)."
    ));
}

#[test]
fn regression_requires_weighted_decay_abuse_guard_marker() {
    // Regression: #730
    assert!(DOC.contains(
        "replayed reciprocity, burst-spam, and churn abuse fixtures remain penalized (`Regression: #730`)."
    ));
}

#[test]
fn regression_requires_dispute_reason_code_policy_guard_marker() {
    // Regression: #934
    assert!(DOC.contains(
        "reason-code mismatch or tampered dispute evidence payloads force `NO-GO` (`Regression: #934`)."
    ));
}

#[test]
fn regression_requires_signal_quarantine_tamper_guard_marker() {
    // Regression: #935
    assert!(DOC.contains(
        "tampered quarantine reason codes, replayed nonces, and stale signal payloads force `NO-GO` quarantine (`Regression: #935`)."
    ));
}

#[test]
fn regression_requires_recovery_irreversible_penalty_guard_marker() {
    // Regression: #936
    assert!(DOC.contains(
        "false-positive irreversible-penalty paths, replayed recovery nonces, and tampered recovery reason codes force `NO-GO` (`Regression: #936`)."
    ));
}
