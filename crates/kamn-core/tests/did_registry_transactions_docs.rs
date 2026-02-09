const DOC: &str = include_str!("../../../docs/foundation/did-registry-transactions.md");

#[test]
fn doc_contains_retry_classification_contract() {
    assert!(DOC.contains("idempotency_key_for_register"));
    assert!(DOC.contains("submit_registration_via_chain_adapter"));
    assert!(DOC.contains("register_with_retry_guard"));
    assert!(DOC.contains("NewSubmission"));
    assert!(DOC.contains("RetryableInFlight"));
    assert!(DOC.contains("FinalizedNoRetry"));
    assert!(DOC.contains("ConflictNoRetry"));
}

#[test]
fn doc_contains_finality_safety_rules() {
    assert!(DOC.contains("Submitted(receipt)"));
    assert!(DOC.contains("Duplicate(receipt)"));
    assert!(DOC.contains("Rejected { reason }"));
    assert!(DOC.contains("FinalizedNoOp"));
    assert!(DOC.contains("InMemoryDidRegistrationChainAdapter"));
    assert!(DOC.contains("record_register_finality"));
    assert!(DOC.contains("StaleFinalityUpdate"));
    assert!(DOC.contains("ConflictingFinalityUpdate"));
    assert!(DOC.contains("UnknownSubmissionIdempotencyKey"));
}

#[test]
fn regression_doc_marks_stale_duplicate_conflict_guard() {
    // Regression: #678
    assert!(DOC.contains("Regression: #678"));
}

#[test]
fn doc_contains_lifecycle_mutation_transaction_contracts() {
    assert!(DOC.contains("DidLifecycleMutationRequest"));
    assert!(DOC.contains("apply_lifecycle_mutation"));
    assert!(DOC.contains("did_lifecycle_mutation_allowed"));
    assert!(DOC.contains("did_lifecycle_mutation_nonce_replay"));
    assert!(DOC.contains("did_lifecycle_mutation_unauthorized_actor"));
}

#[test]
fn regression_doc_marks_lifecycle_mutation_fail_closed_guard() {
    // Regression: #889
    assert!(DOC.contains("Regression: #889"));
}
