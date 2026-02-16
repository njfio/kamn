const DOC: &str = include_str!("../../../docs/foundation/message-proof-anchoring.md");

#[test]
fn doc_contains_anchor_service_contracts() {
    assert!(DOC.contains("MessageProofAnchoringService"));
    assert!(DOC.contains("anchor_message_proof_via_chain_adapter"));
    assert!(DOC.contains("idempotency_key_for_anchor"));
    assert!(DOC.contains("NewSubmission"));
    assert!(DOC.contains("RetryableInFlight"));
    assert!(DOC.contains("FinalizedNoRetry"));
    assert!(DOC.contains("ConflictNoRetry"));
}

#[test]
fn doc_contains_kolme_and_outcome_contracts() {
    assert!(DOC.contains("KolmeMessageProofChainAdapter"));
    assert!(DOC.contains("InMemoryMessageProofChainAdapter"));
    assert!(DOC.contains("Submitted(receipt)"));
    assert!(DOC.contains("Duplicate(receipt)"));
    assert!(DOC.contains("Rejected { reason }"));
    assert!(DOC.contains("FinalizedNoOp"));
}

#[test]
fn regression_doc_marks_conflicting_idempotency_fail_closed_guard() {
    // Regression: #2941
    assert!(DOC.contains("Regression: #2941"));
}

#[test]
fn regression_doc_marks_mismatch_tamper_reason_taxonomy_and_ci_boundary_contracts() {
    // Regression: #4419
    assert!(DOC.contains("Regression: #4419"));
    assert!(DOC.contains(
        "anchoring_gate_reason_taxonomy_version=kamn.kolme.message-proof-anchoring-gate-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "anchoring_gate_reason_codes_csv=message_anchor_evidence_mismatch,message_anchor_evidence_tamper_detected,message_proof_anchor_conflicting_key,message_proof_anchor_invalid_state,ci_fast_gate_failed,local_heavy_opt_in_required"
    ));
    assert!(DOC.contains("ci_smoke_local_heavy_boundary_status=verified"));
}
