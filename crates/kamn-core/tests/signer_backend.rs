use kamn_core::{
    baseline_signature_for_fields, BaselineTransaction, SignerBackendError, SignerBackendRouter,
    SignerProviderHandshakeMatrix, SignerProviderHandshakeStatus, SigningRequest,
    TransactionGuards, GENESIS_STATE_HASH,
};
use std::time::Instant;

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
fn functional_aws_kms_provider_routes_to_production_adapter_backend() {
    let router = SignerBackendRouter::default();
    let request = SigningRequest::new(
        "secure:aws-kms:key-ops-1",
        "agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");

    let signed = router
        .sign_with_secure_fallback(&request)
        .expect("secure backend should sign");
    assert_eq!(signed.backend, "secure-aws-kms-emulator");

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
fn functional_provider_handshake_matrix_routes_operator_fallback_for_unavailable_provider() {
    let router = SignerBackendRouter::with_provider_handshake_matrix(
        SignerProviderHandshakeMatrix::with_statuses(
            SignerProviderHandshakeStatus::Available,
            SignerProviderHandshakeStatus::Unavailable,
        ),
    );
    let request = SigningRequest::new(
        "secure:aws-kms:key-ops-1",
        "agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");

    let signed = router
        .sign_with_secure_fallback(&request)
        .expect("operator fallback should sign when provider is unavailable");
    assert_eq!(signed.backend, "local-software");
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
fn integration_aws_kms_signed_transaction_passes_transaction_guards() {
    let router = SignerBackendRouter::default();
    let mut guards = TransactionGuards::new();
    let mut tx = BaselineTransaction::signed(
        "tx-sign-aws-1",
        "agent-a",
        1,
        "payload-sign-1",
        guards.expected_state_hash(),
    );

    let request = SigningRequest::for_transaction("secure:aws-kms:key-ops-2", &tx)
        .expect("request should map from transaction");
    let signed = router
        .sign_with_secure_fallback(&request)
        .expect("signing should succeed");
    assert_eq!(signed.backend, "secure-aws-kms-emulator");
    tx.signature = signed.signature;

    guards
        .validate_and_record(&tx)
        .expect("signed transaction should validate");
}

#[test]
fn functional_admin_role_key_signs_when_sender_role_matches() {
    let router = SignerBackendRouter::default();
    let request = SigningRequest::new(
        "secure:aws-kms:role-admin/key-ops-1",
        "admin-agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");

    let signed = router
        .sign_with_secure_fallback(&request)
        .expect("signing should succeed");
    assert_eq!(signed.backend, "secure-aws-kms-emulator");

    router
        .verify_with_backend(&signed.backend, &request, &signed.signature)
        .expect("signature should verify");
}

#[test]
fn regression_role_mismatch_signing_request_is_rejected() {
    // Regression: #619
    let router = SignerBackendRouter::default();
    let request = SigningRequest::new(
        "secure:aws-kms:role-admin/key-ops-1",
        "agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");

    assert_eq!(
        router.sign_with_secure_fallback(&request),
        Err(SignerBackendError::KeyRoleMismatch {
            key_role: "admin".to_owned(),
            sender_role: "operator".to_owned(),
            sender: "agent-a".to_owned(),
            key_id: "secure:aws-kms:role-admin/key-ops-1".to_owned(),
        })
    );
}

#[test]
fn regression_admin_key_does_not_fallback_when_secure_provider_unavailable() {
    // Regression: #619
    let router = SignerBackendRouter::with_secure_availability(false);
    let request = SigningRequest::new(
        "secure:aws-kms:role-admin/key-ops-1",
        "admin-agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");

    assert_eq!(
        router.sign_with_secure_fallback(&request),
        Err(SignerBackendError::FallbackDeniedByRolePolicy {
            key_role: "admin".to_owned(),
            key_id: "secure:aws-kms:role-admin/key-ops-1".to_owned(),
        })
    );
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
fn regression_unknown_secure_provider_is_rejected_without_fallback() {
    // Regression: #619
    let router = SignerBackendRouter::default();
    let request = SigningRequest::new(
        "secure:gcp-kms:key-ops-1",
        "agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");

    assert_eq!(
        router.sign_with_secure_fallback(&request),
        Err(SignerBackendError::UnsupportedSecureProvider {
            backend: "secure-mock".to_owned(),
            provider: "gcp-kms".to_owned(),
            key_id: "secure:gcp-kms:key-ops-1".to_owned(),
        })
    );
}

#[test]
fn regression_provider_handshake_policy_block_rejects_without_fallback() {
    // Regression: #677
    let router = SignerBackendRouter::with_provider_handshake_matrix(
        SignerProviderHandshakeMatrix::with_statuses(
            SignerProviderHandshakeStatus::Available,
            SignerProviderHandshakeStatus::PolicyBlocked,
        ),
    );
    let request = SigningRequest::new(
        "secure:aws-kms:key-ops-1",
        "agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");

    assert_eq!(
        router.sign_with_secure_fallback(&request),
        Err(SignerBackendError::ProviderHandshakeRejected {
            backend: "secure-aws-kms-emulator".to_owned(),
            failure_class: "policy-blocked".to_owned(),
        })
    );
}

#[test]
fn regression_secure_provider_backend_mismatch_is_rejected() {
    // Regression: #619
    let router = SignerBackendRouter::default();
    let request = SigningRequest::new(
        "secure:aws-kms:key-ops-1",
        "agent-a",
        1,
        "payload-1",
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid");
    let signed = router
        .sign_with_secure_fallback(&request)
        .expect("signing should succeed");

    assert_eq!(
        router.verify_with_backend("secure-mock", &request, &signed.signature),
        Err(SignerBackendError::SecureProviderBackendMismatch {
            expected_backend: "secure-aws-kms-emulator".to_owned(),
            provided_backend: "secure-mock".to_owned(),
            key_id: "secure:aws-kms:key-ops-1".to_owned(),
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

#[test]
fn performance_signer_emulator_contract_lane_stays_within_budget() {
    let router = SignerBackendRouter::default();
    let start = Instant::now();

    for nonce in 1..=256 {
        let key_id = if nonce % 2 == 0 {
            "secure:key-ops-perf"
        } else {
            "secure:aws-kms:key-ops-perf"
        };
        let request = SigningRequest::new(
            key_id,
            "agent-a",
            nonce,
            &format!("payload-perf-{nonce}"),
            GENESIS_STATE_HASH,
        )
        .expect("request should be valid");
        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("signature should be produced");
        let expected_backend = if nonce % 2 == 0 {
            "secure-mock"
        } else {
            "secure-aws-kms-emulator"
        };
        assert_eq!(signed.backend, expected_backend);
    }

    let elapsed_millis = start.elapsed().as_millis();
    assert!(
        elapsed_millis < 250,
        "signer emulator contract lane exceeded budget: {elapsed_millis}ms"
    );
}

#[test]
#[ignore = "scheduled provider integration lane"]
fn performance_signer_emulator_bulk_signing_deep_lane() {
    let secure_router = SignerBackendRouter::default();
    let fallback_router = SignerBackendRouter::with_secure_availability(false);

    for nonce in 1..=5000 {
        let request = SigningRequest::new(
            "secure:key-ops-deep",
            "agent-a",
            nonce,
            &format!("payload-deep-{nonce}"),
            GENESIS_STATE_HASH,
        )
        .expect("request should be valid");

        let (router, expected_backend) = if nonce % 10 == 0 {
            (&fallback_router, "local-software")
        } else {
            (&secure_router, "secure-mock")
        };

        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("signature should be produced");
        assert_eq!(signed.backend, expected_backend);
    }
}
