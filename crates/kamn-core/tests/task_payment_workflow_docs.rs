const DOC: &str = include_str!("../../../docs/foundation/task-payment-workflow.md");

#[test]
fn doc_contains_payment_offer_confirm_models_and_workflow_contract() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("PaymentOffer"));
    assert!(DOC.contains("PaymentConfirm"));
    assert!(DOC.contains("TaskPaymentWorkflow"));
    assert!(DOC.contains("TaskPaymentError"));
}

#[test]
fn doc_contains_completion_and_escrow_validation_rules() {
    assert!(DOC.contains("## Deterministic Validation Rules"));
    assert!(DOC.contains("target task in `Completed` state."));
    assert!(DOC.contains("`payer_did` equal to task requester DID."));
    assert!(DOC.contains("`payee_did` equal to task assignee DID."));
    assert!(DOC.contains("offered amount less than or equal to escrow remaining balance."));
    assert!(DOC.contains("single-use confirmation (duplicates are rejected)."));
    assert!(DOC.contains("current_unix >= timeout_unix"));
    assert!(DOC.contains("## Escrow Release Behavior"));
    assert!(DOC.contains("EscrowLifecycle::refund_after_timeout(current_unix, timeout_unix)"));
}

#[test]
fn regression_requires_duplicate_confirm_rejection_rule() {
    // Regression: #216
    assert!(DOC.contains("duplicates are rejected"));
}

#[test]
fn regression_requires_premature_timeout_refund_rejection_rule() {
    // Regression: #542
    assert!(DOC.contains("premature timeout refund attempts are rejected (`Regression: #542`)."));
}

#[test]
fn regression_requires_participant_binding_rules() {
    // Regression: #558
    assert!(DOC.contains("`payer_did` equal to task requester DID."));
    assert!(DOC.contains("`payee_did` equal to task assignee DID."));
}

#[test]
fn doc_contains_settlement_evidence_schema_and_checker_contract() {
    assert!(DOC.contains("## Settlement Evidence Schema and Policy Checks"));
    assert!(DOC.contains("generate_settlement_reconciliation_evidence_bundle.sh"));
    assert!(DOC.contains("check_settlement_reconciliation_evidence_policy.sh"));
    assert!(DOC.contains("settlement_reconciliation_reason_codes:GO:v1"));
    assert!(DOC.contains("settlement_reconciliation_reason_codes:NO-GO:v1"));
}

#[test]
fn regression_requires_settlement_evidence_schema_drift_fail_closed_rule() {
    // Regression: #906
    assert!(DOC.contains("settlement evidence schema drift must fail closed (`Regression: #906`)."));
}
