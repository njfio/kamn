const DOC: &str = include_str!("../../../docs/foundation/did-registry-transactions.md");

#[test]
fn doc_contains_retry_classification_contract() {
    assert!(DOC.contains("idempotency_key_for_register"));
    assert!(DOC.contains("register_with_retry_guard"));
    assert!(DOC.contains("NewSubmission"));
    assert!(DOC.contains("RetryableInFlight"));
    assert!(DOC.contains("FinalizedNoRetry"));
    assert!(DOC.contains("ConflictNoRetry"));
}

#[test]
fn doc_contains_finality_safety_rules() {
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
