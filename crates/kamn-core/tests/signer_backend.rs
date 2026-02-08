use kamn_core::{
    baseline_signature_for_fields, BaselineTransaction, SignerBackendError, SignerBackendRouter,
    SigningRequest, TransactionGuards, GENESIS_STATE_HASH,
};

#[test]
fn functional_secure_backend_signs_and_verifies_when_available() {
    let router = SignerBackendRouter::default();
    let request = SigningRequest::new(
        "secure:key-ops-1",
        "agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");

    let signed = router
        .sign_with_secure_fallback(&request)
        .expect("secure backend should sign");
    assert_eq!(signed.backend, "secure-mock");

    router
        .verify_with_backend(&signed.backend, &request, &signed.signature)
        .expect("signature should verify");
}

#[test]
fn functional_secure_unavailable_falls_back_to_local_backend() {
    let router = SignerBackendRouter::with_secure_availability(false);
    let request = SigningRequest::new(
        "secure:key-ops-1",
        "agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");

    let signed = router
        .sign_with_secure_fallback(&request)
        .expect("fallback should sign");
    assert_eq!(signed.backend, "local-software");

    router
        .verify_with_backend(&signed.backend, &request, &signed.signature)
        .expect("fallback signature should verify");
}

#[test]
fn integration_router_signed_transaction_passes_transaction_guards() {
    let router = SignerBackendRouter::default();
    let mut guards = TransactionGuards::new();
    let mut tx = BaselineTransaction::signed(
        "tx-sign-1",
        "agent-a",
        1,
        "payload-sign-1",
        guards.expected_state_hash(),
    );

    let request = SigningRequest::for_transaction("secure:key-ops-2", &tx)
        .expect("request should map from transaction");
    let signed = router
        .sign_with_secure_fallback(&request)
        .expect("signing should succeed");
    tx.signature = signed.signature;

    guards
        .validate_and_record(&tx)
        .expect("signed transaction should validate");
}

#[test]
fn regression_unsupported_secure_key_reference_does_not_fallback() {
    // Regression: #160
    let router = SignerBackendRouter::default();
    let request = SigningRequest::new("local:key-1", "agent-a", 1, "payload-1", GENESIS_STATE_HASH)
        .expect("request should be valid");

    assert_eq!(
        router.sign_with_secure_fallback(&request),
        Err(SignerBackendError::UnsupportedKeyReference {
            backend: "secure-mock".to_owned(),
            key_id: "local:key-1".to_owned(),
        })
    );
}

#[test]
fn for_transaction_rejects_empty_transaction_id() {
    let tx = BaselineTransaction {
        id: String::new(),
        sender: "agent-a".to_owned(),
        nonce: 1,
        payload: "payload-1".to_owned(),
        state_hash: GENESIS_STATE_HASH.to_owned(),
        signature: "sig:placeholder".to_owned(),
    };

    assert_eq!(
        SigningRequest::for_transaction("secure:key-ops-3", &tx),
        Err(SignerBackendError::EmptyField("transaction_id"))
    );
}

#[test]
fn regression_signing_request_matches_canonical_signature_profile() {
    // Regression: #400
    let router = SignerBackendRouter::default();
    let request = SigningRequest::new(
        "secure:key-ops-1",
        "agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");

    let signed = router
        .sign_with_secure_fallback(&request)
        .expect("signature should be produced");
    let canonical = baseline_signature_for_fields("agent-a", 1, GENESIS_STATE_HASH, "payload-1");
    assert_eq!(signed.signature, canonical);
}

#[test]
fn regression_signatures_include_profile_identifier_segment() {
    // Regression: #404
    let router = SignerBackendRouter::default();
    let request = SigningRequest::new(
        "secure:key-ops-1",
        "agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");

    let signed = router
        .sign_with_secure_fallback(&request)
        .expect("signature should be produced");
    assert!(signed.signature.starts_with("sig:baseline-v1:"));
}
