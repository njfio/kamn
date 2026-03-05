use super::*;

const CORE_DEFAULT_SENDER: &str = "agent-a";
const CORE_DEFAULT_NONCE: u64 = 1;
const CORE_DEFAULT_PAYLOAD: &str = "payload-1";
const CORE_SECURE_KEY_ID: &str = "secure:key-ops-1";
const CORE_AWS_KEY_ID: &str = "secure:aws-kms:key-ops-1";
const CORE_LOCAL_UNSUPPORTED_KEY_ID: &str = "local:key-1";
const CORE_MISSING_KEY_ID: &str = "secure:key-regression-5913-missing";
const CORE_TX_KEY_ID: &str = "secure:key-ops-2";
const CORE_AWS_TX_KEY_ID: &str = "secure:aws-kms:key-ops-2";
const CORE_TX_ID: &str = "tx-sign-1";
const CORE_AWS_TX_ID: &str = "tx-sign-aws-1";
const CORE_TX_PAYLOAD: &str = "payload-sign-1";
const CORE_SIGNER_KEY_ENV: &str = "KAMN_SIGNER_PRIVATE_KEY_HEX";
const CORE_SERVICE_SIGNER_KEY_ENV: &str = "KAMN_SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_HEX";
const CORE_MISSING_KEY_ENV: &str = "KAMN_SIGNER_PRIVATE_KEY_HEX__SECURE_KEY_REGRESSION_5913_MISSING";

fn signer_core_request(key_id: &str, payload: &str) -> SigningRequest {
    SigningRequest::new(
        key_id,
        CORE_DEFAULT_SENDER,
        CORE_DEFAULT_NONCE,
        payload,
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid")
}

fn signer_core_signed_transaction(
    transaction_id: &str,
    payload: &str,
    guards: &TransactionGuards,
) -> BaselineTransaction {
    BaselineTransaction::signed(
        transaction_id,
        CORE_DEFAULT_SENDER,
        CORE_DEFAULT_NONCE,
        payload,
        guards.expected_state_hash(),
    )
}

fn signer_core_custom_provider_client(
    request: &SigningRequest,
    key_reference: &CanonicalSecureKeyReference,
) -> Result<BackendSignature, SignerBackendError> {
    Ok(BackendSignature {
        backend: key_reference.provider.backend_name().to_owned(),
        signature: format!(
            "provider-client:{}",
            baseline_signature_for_fields(
                &request.sender,
                request.nonce,
                &request.state_hash,
                &request.payload,
            )
        ),
    })
}

pub(super) fn run_functional_secure_backend_signs_and_verifies_when_available() {
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::default();
        let request = signer_core_request(CORE_SECURE_KEY_ID, CORE_DEFAULT_PAYLOAD);
        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("secure backend should sign");
        assert_eq!(signed.backend, "secure-mock");
        router
            .verify_with_backend(&signed.backend, &request, &signed.signature)
            .expect("signature should verify");
    });
}

pub(super) fn run_functional_aws_kms_provider_routes_to_production_adapter_backend() {
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::default();
        let request = signer_core_request(CORE_AWS_KEY_ID, CORE_DEFAULT_PAYLOAD);
        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("secure backend should sign");
        assert_eq!(signed.backend, "secure-aws-kms-emulator");
        router
            .verify_with_backend(&signed.backend, &request, &signed.signature)
            .expect("signature should verify");
    });
}

pub(super) fn run_functional_router_uses_custom_provider_client_mapping_for_secure_provider() {
    let router = SignerBackendRouter::with_provider_client(
        SignerProviderHandshakeMatrix::with_uniform_availability(true),
        signer_core_custom_provider_client,
    );
    let request = signer_core_request(CORE_AWS_KEY_ID, CORE_DEFAULT_PAYLOAD);
    let signed = router
        .sign_with_secure_fallback(&request)
        .expect("provider client should sign through secure router");
    assert_eq!(signed.backend, "secure-aws-kms-emulator");
    assert!(signed
        .signature
        .starts_with("provider-client:sig:deterministic-v1:baseline-v1"));
}

pub(super) fn run_functional_secure_unavailable_falls_back_to_local_backend() {
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::with_secure_availability(false);
        let request = signer_core_request(CORE_SECURE_KEY_ID, CORE_DEFAULT_PAYLOAD);
        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("fallback should sign");
        assert_eq!(signed.backend, "local-software");
        router
            .verify_with_backend(&signed.backend, &request, &signed.signature)
            .expect("fallback signature should verify");
    });
}

pub(super) fn run_regression_local_backend_signing_requires_explicit_key_material() {
    // Regression: #5913
    let _lock = signer_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _generic_key_guard = EnvVarGuard::set(CORE_SIGNER_KEY_ENV, None);
    let _service_key_guard = EnvVarGuard::set(CORE_SERVICE_SIGNER_KEY_ENV, None);
    let _key_specific_guard = EnvVarGuard::set(CORE_MISSING_KEY_ENV, None);

    let router = SignerBackendRouter::with_secure_availability(false);
    let request = signer_core_request(CORE_MISSING_KEY_ID, CORE_DEFAULT_PAYLOAD);
    let result = router.sign_with_secure_fallback(&request);
    assert!(
        matches!(
            result,
            Err(SignerBackendError::MissingSigningKeyMaterial { key_id, .. })
                if key_id == CORE_MISSING_KEY_ID
        ),
        "local signing must fail closed when signer key env is not provisioned"
    );
}

pub(super) fn run_integration_router_signed_transaction_passes_transaction_guards() {
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::default();
        let mut guards = TransactionGuards::new();
        let mut tx = signer_core_signed_transaction(CORE_TX_ID, CORE_TX_PAYLOAD, &guards);
        let request = SigningRequest::for_transaction(CORE_TX_KEY_ID, &tx)
            .expect("request should map from transaction");
        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("signing should succeed");
        tx.signature = signed.signature;
        guards
            .validate_and_record(&tx)
            .expect("signed transaction should validate");
    });
}

pub(super) fn run_integration_aws_kms_signed_transaction_passes_transaction_guards() {
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::default();
        let mut guards = TransactionGuards::new();
        let mut tx = signer_core_signed_transaction(CORE_AWS_TX_ID, CORE_TX_PAYLOAD, &guards);
        let request = SigningRequest::for_transaction(CORE_AWS_TX_KEY_ID, &tx)
            .expect("request should map from transaction");
        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("signing should succeed");
        assert_eq!(signed.backend, "secure-aws-kms-emulator");
        tx.signature = signed.signature;
        guards
            .validate_and_record(&tx)
            .expect("signed transaction should validate");
    });
}

pub(super) fn run_regression_unsupported_secure_key_reference_does_not_fallback() {
    // Regression: #160
    let router = SignerBackendRouter::default();
    let request = signer_core_request(CORE_LOCAL_UNSUPPORTED_KEY_ID, CORE_DEFAULT_PAYLOAD);
    assert_eq!(
        router.sign_with_secure_fallback(&request),
        Err(SignerBackendError::UnsupportedKeyReference {
            backend: "secure-mock".to_owned(),
            key_id: CORE_LOCAL_UNSUPPORTED_KEY_ID.to_owned(),
        })
    );
}
