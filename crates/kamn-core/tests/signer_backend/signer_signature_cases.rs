use super::*;

const SIGNATURE_COMPAT_ENV_KEY: &str = "KAMN_SIGNER_ALLOW_LEGACY_BASELINE_V1";
const SIGNATURE_WRONG_KEY_ENV_KEY: &str =
    "KAMN_SIGNER_PRIVATE_KEY_HEX__SECURE_KEY_REGRESSION_5897_WRONG_KEY";
const SIGNATURE_DEFAULT_KEY_ID: &str = "secure:key-ops-1";
const SIGNATURE_WRONG_KEY_ID: &str = "secure:key-regression-5897-wrong-key";
const SIGNATURE_DEFAULT_SENDER: &str = "agent-a";
const SIGNATURE_DEFAULT_NONCE: u64 = 1;
const SIGNATURE_DEFAULT_PAYLOAD: &str = "payload-1";

fn with_signature_compat_env<T>(value: Option<&str>, run: impl FnOnce() -> T) -> T {
    let _lock = signer_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _compat_guard = EnvVarGuard::set(SIGNATURE_COMPAT_ENV_KEY, value);
    run()
}

fn signer_signature_request(key_id: &str) -> SigningRequest {
    SigningRequest::new(
        key_id,
        SIGNATURE_DEFAULT_SENDER,
        SIGNATURE_DEFAULT_NONCE,
        SIGNATURE_DEFAULT_PAYLOAD,
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid")
}

fn baseline_v1_signature() -> String {
    baseline_signature_for_fields(
        SIGNATURE_DEFAULT_SENDER,
        SIGNATURE_DEFAULT_NONCE,
        GENESIS_STATE_HASH,
        SIGNATURE_DEFAULT_PAYLOAD,
    )
}

fn signer_local_fallback_router() -> SignerBackendRouter {
    SignerBackendRouter::with_secure_availability(false)
}

fn assert_baseline_v1_rejected_by_backend(backend: &str, message: &str) {
    let router = SignerBackendRouter::default();
    let request = signer_signature_request(SIGNATURE_DEFAULT_KEY_ID);
    let signature = baseline_v1_signature();
    assert!(
        router
            .verify_with_backend(backend, &request, signature.as_str())
            .is_err(),
        "{message}"
    );
}

pub(super) fn run_integration_signer_backend_accepts_baseline_v1_only_with_explicit_compatibility_switch(
) {
    with_signature_compat_env(Some("1"), || {
        let router = SignerBackendRouter::default();
        let request = signer_signature_request(SIGNATURE_DEFAULT_KEY_ID);
        let signature = baseline_v1_signature();
        assert!(
            router
                .verify_with_backend("secure-mock", &request, signature.as_str())
                .is_ok(),
            "baseline-v1 signatures should be accepted only when explicit compatibility switch is enabled"
        );
    });
}

pub(super) fn run_regression_signer_backend_rejects_baseline_v1_signature_by_default() {
    // Regression: #5897
    assert_baseline_v1_rejected_by_backend(
        "secure-mock",
        "baseline-v1 signatures must be rejected by default",
    );
}

pub(super) fn run_regression_local_backend_rejects_tampered_signature() {
    // Regression: #5897
    with_default_signer_key_env(|| {
        let router = signer_local_fallback_router();
        let request = signer_signature_request(SIGNATURE_DEFAULT_KEY_ID);
        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("local fallback should sign");
        assert_eq!(signed.backend, "local-software");
        let tampered_signature = format!("{}ff", signed.signature);
        assert!(
            router
                .verify_with_backend("local-software", &request, tampered_signature.as_str())
                .is_err(),
            "local backend must reject tampered signatures"
        );
    });
}

pub(super) fn run_regression_local_backend_rejects_signature_when_verifier_uses_wrong_key() {
    // Regression: #5897
    with_signature_compat_env(None, || {
        let router = signer_local_fallback_router();
        let request = signer_signature_request(SIGNATURE_WRONG_KEY_ID);
        let _signing_key_guard = EnvVarGuard::set(
            SIGNATURE_WRONG_KEY_ENV_KEY,
            Some(TEST_SIGNER_PRIVATE_KEY_A_HEX),
        );
        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("local fallback should sign");
        let _verifying_key_guard = EnvVarGuard::set(
            SIGNATURE_WRONG_KEY_ENV_KEY,
            Some(TEST_SIGNER_PRIVATE_KEY_B_HEX),
        );
        assert!(
            router
                .verify_with_backend("local-software", &request, signed.signature.as_str())
                .is_err(),
            "local backend must reject signatures when verifier key material does not match signer key"
        );
    });
}

pub(super) fn run_regression_local_backend_rejects_baseline_v1_signature_without_compat_switch() {
    // Regression: #5897
    assert_baseline_v1_rejected_by_backend(
        "local-software",
        "baseline-v1 must not bypass local backend verification when compat switch is disabled",
    );
}
