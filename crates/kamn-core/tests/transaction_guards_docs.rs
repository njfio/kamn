const DOC: &str = include_str!("../../../docs/foundation/transaction-guards.md");

#[test]
fn doc_contains_transaction_guard_scope_and_components() {
    assert!(DOC.contains("## Invariants Enforced"));
    assert!(DOC.contains("BaselineTransaction"));
    assert!(DOC.contains("TransactionGuards"));
    assert!(DOC.contains("TransactionGuardError"));
}

#[test]
fn doc_contains_canonical_signature_profile_contract() {
    assert!(DOC.contains("## Canonical Signature Profile"));
    assert!(DOC.contains("baseline_signature_for_fields(...)"));
    assert!(DOC.contains("shared between `transaction` and `signer_backend` paths"));
    assert!(DOC.contains("baseline signature profile id: `baseline-v1`"));
}

#[test]
fn regression_requires_signature_profile_drift_guard_rule() {
    // Regression: #400
    assert!(DOC.contains(
        "signature-profile drift between transaction and signer paths is rejected (`Regression: #400`).",
    ));
    assert!(DOC.contains("non-versioned signature profile is rejected (`Regression: #404`)."));
}
