const DOC: &str = include_str!("../../../docs/foundation/escrow-lifecycle.md");

#[test]
fn doc_contains_escrow_lifecycle_scope_and_transitions() {
    assert!(DOC.contains("# Escrow Lifecycle State Machine"));
    assert!(DOC.contains("EscrowLifecycle"));
    assert!(DOC.contains("Disputed -> Resolved"));
}

#[test]
fn doc_contains_settlement_reconciliation_evidence_contract() {
    assert!(DOC.contains("## Settlement Reconciliation Evidence Contract"));
    assert!(DOC.contains("generate_settlement_reconciliation_evidence_bundle.sh"));
    assert!(DOC.contains("check_settlement_reconciliation_evidence_policy.sh"));
    assert!(DOC.contains("run_settlement_reconciliation_contract_lane.sh"));
    assert!(DOC.contains("run_settlement_reconciliation_deep_lane.sh"));
}

#[test]
fn regression_requires_missing_receipt_evidence_guard_marker() {
    // Regression: #678
    assert!(DOC.contains(
        "missing or invalid chain receipt evidence forces `NO-GO` (`Regression: #678`).",
    ));
}
