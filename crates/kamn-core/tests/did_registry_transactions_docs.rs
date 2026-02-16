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
    assert!(DOC.contains("submit_lifecycle_mutation_via_chain_adapter"));
    assert!(DOC.contains("KolmeDidLifecycleChainAdapter"));
    assert!(DOC.contains("record_lifecycle_finality"));
    assert!(DOC.contains("did_lifecycle_mutation_allowed"));
    assert!(DOC.contains("did_lifecycle_mutation_nonce_replay"));
    assert!(DOC.contains("did_lifecycle_mutation_unauthorized_actor"));
    assert!(DOC.contains("did_chain_adapter_submit_failed"));
}

#[test]
fn regression_doc_marks_lifecycle_mutation_fail_closed_guard() {
    // Regression: #889
    assert!(DOC.contains("Regression: #889"));
}

#[test]
fn regression_doc_marks_lifecycle_chain_submission_conflict_guard() {
    // Regression: #2936
    assert!(DOC.contains("Regression: #2936"));
}

#[test]
fn regression_doc_marks_registration_payload_integrity_and_duplicate_drift_guards() {
    // Regression: #4418
    assert!(DOC.contains("Regression: #4418"));
    assert!(DOC.contains(
        "did_registration_reason_taxonomy_version=kamn.kolme.did-registration-reason-taxonomy.v1"
    ));
    assert!(DOC.contains("did_registration_reason_codes_csv=did_registry_document_did_mismatch,did_registry_submission_key_conflict"));
}
