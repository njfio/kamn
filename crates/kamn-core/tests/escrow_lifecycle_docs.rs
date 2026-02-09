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
    assert!(DOC.contains("--ledger-reference-id"));
}

#[test]
fn doc_contains_chain_receipt_finality_adapter_contract() {
    assert!(DOC.contains("## Chain Receipt Finality Adapter Contract"));
    assert!(DOC.contains("EscrowReceiptFinality::{Final, Pending, Failed}"));
    assert!(DOC.contains("reconcile_receipt_finality(receipt_id, finality, action)"));
    assert!(DOC.contains("EscrowSettlementOutcome::{Settled, Pending, Rejected}"));
    assert!(DOC.contains("EscrowReceiptFinality::parse(...)"));
}

#[test]
fn doc_contains_transition_evidence_reason_code_contract() {
    assert!(DOC.contains("## Transition Evidence and Reason-Code Contract (Issue #903)"));
    assert!(DOC.contains("apply_transition_with_evidence"));
    assert!(DOC.contains("EscrowTransitionEvidence"));
    assert!(DOC.contains("escrow_transition_allowed"));
    assert!(DOC.contains("escrow_transition_invalid"));
    assert!(DOC.contains("escrow_settlement_finalized"));
}

#[test]
fn doc_contains_timeout_finality_race_matrix_contract() {
    assert!(DOC.contains("## Timeout/Finality Race Matrix Evidence"));
    assert!(DOC.contains("run_settlement_reconciliation_race_matrix.py"));
    assert!(DOC.contains("fixtures/escrow_reconciliation/finality_race_cases.json"));
}

#[test]
fn regression_requires_missing_receipt_evidence_guard_marker() {
    // Regression: #678
    assert!(DOC.contains(
        "missing or invalid chain receipt evidence forces `NO-GO` (`Regression: #678`).",
    ));
}

#[test]
fn regression_requires_ledger_reference_guard_marker() {
    // Regression: #717
    assert!(DOC.contains(
        "missing ledger reference evidence and ledger amount drift force `NO-GO` (`Regression: #717`).",
    ));
}

#[test]
fn regression_requires_transition_reason_code_guard_marker() {
    // Regression: #903
    assert!(DOC.contains(
        "transition reason-code drift and illegal transition acceptance fail closed (`Regression: #903`).",
    ));
}
