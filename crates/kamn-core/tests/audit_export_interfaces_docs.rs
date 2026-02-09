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
