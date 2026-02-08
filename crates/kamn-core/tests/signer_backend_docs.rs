const DOC: &str = include_str!("../../../docs/foundation/signer-backend-abstraction.md");

#[test]
fn doc_contains_signer_backend_contract_and_router_rules() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("SigningRequest"));
    assert!(DOC.contains("SignerBackend"));
    assert!(DOC.contains("LocalSignerBackend"));
    assert!(DOC.contains("SecureSignerBackend"));
    assert!(DOC.contains("sign_with_secure_fallback"));
}

#[test]
fn doc_contains_fallback_semantics_and_transaction_integration() {
    assert!(DOC.contains("## Backend Compatibility Rules"));
    assert!(DOC.contains("falls back from secure to local only for `ProviderUnavailable`."));
    assert!(DOC.contains("does not fallback on hard request errors"));
    assert!(DOC.contains("## Transaction Path Integration"));
    assert!(DOC.contains("SigningRequest::for_transaction(...)"));
    assert!(DOC.contains("baseline_signature_for_fields(...)"));
}

#[test]
fn regression_requires_no_fallback_on_unsupported_secure_key_reference() {
    // Regression: #160
    assert!(DOC.contains("does not fallback on hard request errors"));
    assert!(DOC.contains("canonical signature-profile helper consumed by both paths"));
}
