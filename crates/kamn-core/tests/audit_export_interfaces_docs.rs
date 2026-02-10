const DOC: &str = include_str!("../../../docs/foundation/audit-export-interfaces.md");

#[test]
fn doc_contains_escrow_ledger_reconciliation_evidence_contract() {
    assert!(DOC.contains("## Escrow-Ledger Reconciliation Evidence Contract"));
    assert!(DOC.contains("generate_settlement_reconciliation_evidence_bundle.sh"));
    assert!(DOC.contains("check_settlement_reconciliation_evidence_policy.sh"));
    assert!(DOC.contains("run_settlement_reconciliation_contract_lane.sh"));
    assert!(DOC.contains("run_settlement_reconciliation_deep_lane.sh"));
}

#[test]
fn regression_requires_ledger_reference_evidence_guard_marker() {
    // Regression: #717
    assert!(DOC.contains(
        "missing ledger reference evidence and ledger amount drift force `NO-GO` (`Regression: #717`)."
    ));
}

#[test]
fn doc_contains_soc2_control_evidence_bundle_contract() {
    assert!(DOC.contains("## SOC2 Control Evidence Bundle Contract"));
    assert!(DOC.contains("generate_soc2_control_evidence_bundle.sh"));
    assert!(DOC.contains("check_soc2_control_evidence_policy.sh"));
    assert!(DOC.contains("run_soc2_control_evidence_contract_lane.sh"));
    assert!(DOC.contains("run_soc2_control_evidence_deep_lane.sh"));
    assert!(DOC.contains("run_soc2_control_evidence_replay_matrix.py"));
    assert!(DOC.contains("fixtures/compliance_soc2/control_evidence_replay_cases.json"));
}

#[test]
fn regression_requires_soc2_control_evidence_guard_marker() {
    // Regression: #732
    assert!(DOC.contains(
        "tampered final decisions and incomplete/tampered control evidence force `NO-GO` (`Regression: #732`)."
    ));
}

#[test]
fn doc_contains_dsar_legal_hold_evidence_contract() {
    assert!(DOC.contains("## DSAR Legal-Hold Evidence Contract"));
    assert!(DOC.contains("generate_dsar_legal_hold_evidence_bundle.sh"));
    assert!(DOC.contains("check_dsar_legal_hold_policy.sh"));
    assert!(DOC.contains("run_dsar_legal_hold_contract_lane.sh"));
    assert!(DOC.contains("run_dsar_legal_hold_deep_lane.sh"));
    assert!(DOC.contains("run_dsar_legal_hold_matrix.py"));
    assert!(DOC.contains("fixtures/compliance_dsar/legal_hold_precedence_cases.json"));
}

#[test]
fn regression_requires_dsar_legal_hold_evidence_guard_marker() {
    // Regression: #732
    assert!(DOC.contains(
        "legal-hold bypass attempts and tampered DSAR evidence force `NO-GO` (`Regression: #732`)."
    ));
}

#[test]
fn doc_contains_reputation_dispute_evidence_export_contract() {
    assert!(DOC.contains("## Reputation Dispute Evidence Export Contract"));
    assert!(DOC.contains("generate_reputation_dispute_evidence_bundle.sh"));
    assert!(DOC.contains("check_reputation_dispute_policy.sh"));
    assert!(DOC.contains("run_reputation_dispute_contract_lane.sh"));
    assert!(DOC.contains("run_reputation_dispute_deep_lane.sh"));
    assert!(DOC.contains("run_reputation_dispute_matrix.py"));
    assert!(DOC.contains("reputation_dispute_contract.py"));
    assert!(DOC.contains("fixtures/reputation_dispute/replay_cases.json"));
}

#[test]
fn doc_contains_reputation_signal_quarantine_evidence_export_contract() {
    assert!(DOC.contains("## Reputation Signal Quarantine Evidence Export Contract"));
    assert!(DOC.contains("generate_reputation_signal_quarantine_evidence_bundle.sh"));
    assert!(DOC.contains("check_reputation_signal_quarantine_policy.sh"));
    assert!(DOC.contains("run_reputation_signal_quarantine_contract_lane.sh"));
    assert!(DOC.contains("reputation_signal_quarantine_contract.py"));
}

#[test]
fn doc_contains_reputation_recovery_evidence_export_contract() {
    assert!(DOC.contains("## Reputation Recovery Reversal Evidence Export Contract"));
    assert!(DOC.contains("generate_reputation_recovery_evidence_bundle.sh"));
    assert!(DOC.contains("check_reputation_recovery_policy.sh"));
    assert!(DOC.contains("run_reputation_recovery_contract_lane.sh"));
    assert!(DOC.contains("reputation_recovery_contract.py"));
}

#[test]
fn regression_requires_reputation_dispute_evidence_guard_marker() {
    // Regression: #730
    assert!(DOC.contains(
        "tampered evidence hashes, score-adjustment limit bypasses, and closed-policy-window decisions force `NO-GO` (`Regression: #730`)."
    ));
}

#[test]
fn regression_requires_reputation_signal_quarantine_guard_marker() {
    // Regression: #935
    assert!(DOC.contains(
        "tampered reason keys/reason codes and ingestion-action mismatches force `NO-GO` (`Regression: #935`)."
    ));
}

#[test]
fn regression_requires_reputation_recovery_guard_marker() {
    // Regression: #936
    assert!(DOC.contains(
        "false-positive irreversible-penalty paths, replayed recovery nonces, and tampered recovery reason codes force `NO-GO` (`Regression: #936`)."
    ));
}
