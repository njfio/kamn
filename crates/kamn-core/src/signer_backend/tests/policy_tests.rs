use crate::signer_backend::backends::{
    deterministic_secure_provider_client_sign, SecureSignerBackend, SignerBackend,
};
use crate::signer_backend::env::signer_legacy_baseline_v1_compat_enabled_for_mode_with_env_value;
use crate::signer_backend::provider_policy::{
    CanonicalSecureKeyReference, SecureSignerProvider, SignerProviderHandshakeMatrix,
    SignerProviderHandshakeStatus,
};
use crate::signer_backend::request::SigningRequest;
use crate::signer_backend::router::SignerBackendRouter;
use crate::signer_backend::SignerBackendError;
use std::sync::Mutex;

const TEST_SIGNER_PRIVATE_KEY_A_HEX: &str =
    "7f2dcf2ef6bcf53b1af2359954f04eb6d25688fd87cbf09f7f9db4c6522f4c6b";

fn signer_env_lock() -> &'static Mutex<()> {
    crate::signer_test_env_lock()
}

struct EnvVarGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: Option<&str>) -> Self {
        let previous = std::env::var(key).ok();
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.as_deref() {
            std::env::set_var(self.key, previous);
            return;
        }
        std::env::remove_var(self.key);
    }
}

fn with_default_signer_key_env<T>(run: impl FnOnce() -> T) -> T {
    let _lock = signer_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _generic_key_guard =
        EnvVarGuard::set("KAMN_SIGNER_PRIVATE_KEY_HEX", Some(TEST_SIGNER_PRIVATE_KEY_A_HEX));
    let _service_key_guard = EnvVarGuard::set(
        "KAMN_SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_HEX",
        Some(TEST_SIGNER_PRIVATE_KEY_A_HEX),
    );
    run()
}

#[test]
fn regression_legacy_baseline_compat_helper_fails_closed_for_non_debug_policy() {
    assert!(
        !signer_legacy_baseline_v1_compat_enabled_for_mode_with_env_value(false, Some("1")),
        "non-debug policy branch must not permit legacy baseline compatibility even with truthy env"
    );
}

#[test]
fn regression_legacy_baseline_compat_helper_accepts_truthy_env_for_debug_policy() {
    assert!(
        signer_legacy_baseline_v1_compat_enabled_for_mode_with_env_value(true, Some("1")),
        "debug policy branch should preserve explicit legacy baseline compatibility opt-in"
    );
}

#[test]
fn handshake_matrix_maps_provider_statuses() {
    let matrix = SignerProviderHandshakeMatrix::with_statuses(
        SignerProviderHandshakeStatus::Available,
        SignerProviderHandshakeStatus::PolicyBlocked,
    );
    assert_eq!(
        matrix.status_for_provider(SecureSignerProvider::Mock),
        SignerProviderHandshakeStatus::Available
    );
    assert_eq!(
        matrix.status_for_provider(SecureSignerProvider::AwsKmsEmulator),
        SignerProviderHandshakeStatus::PolicyBlocked
    );
}

#[test]
fn router_decision_matrix_distinguishes_unavailable_vs_policy_blocked_handshakes() {
    with_default_signer_key_env(|| {
        let request = SigningRequest::new(
            "secure:aws-kms:key-ops-1",
            "agent-a",
            1,
            "payload-1",
            "state:genesis",
        )
        .expect("request should be valid");

        let unavailable_router = SignerBackendRouter::with_provider_handshake_matrix(
            SignerProviderHandshakeMatrix::with_statuses(
                SignerProviderHandshakeStatus::Available,
                SignerProviderHandshakeStatus::Unavailable,
            ),
        );
        let signed = unavailable_router
            .sign_with_secure_fallback(&request)
            .expect("unavailable provider should allow operator fallback");
        assert_eq!(signed.backend, "local-software");

        let policy_blocked_router = SignerBackendRouter::with_provider_handshake_matrix(
            SignerProviderHandshakeMatrix::with_statuses(
                SignerProviderHandshakeStatus::Available,
                SignerProviderHandshakeStatus::PolicyBlocked,
            ),
        );
        assert_eq!(
            policy_blocked_router.sign_with_secure_fallback(&request),
            Err(SignerBackendError::ProviderHandshakeRejected {
                backend: "secure-aws-kms-emulator".to_owned(),
                failure_class: "policy-blocked".to_owned(),
            })
        );
    });
}

#[test]
fn secure_backend_rejects_policy_blocked_provider_handshake() {
    let backend = SecureSignerBackend::with_provider_handshake_matrix(
        SignerProviderHandshakeMatrix::with_statuses(
            SignerProviderHandshakeStatus::Available,
            SignerProviderHandshakeStatus::PolicyBlocked,
        ),
    );
    let request = SigningRequest::new(
        "secure:aws-kms:key-prod-1",
        "agent-a",
        1,
        "payload-1",
        "state:genesis",
    )
    .expect("request should be valid");

    assert_eq!(
        backend.sign(&request),
        Err(SignerBackendError::ProviderHandshakeRejected {
            backend: "secure-aws-kms-emulator".to_owned(),
            failure_class: "policy-blocked".to_owned(),
        })
    );
}

#[test]
fn provider_client_maps_backend_from_canonical_reference() {
    with_default_signer_key_env(|| {
        let request = SigningRequest::new(
            "secure:aws-kms:key-prod-1",
            "agent-a",
            1,
            "payload-1",
            "state:genesis",
        )
        .expect("request should be valid");
        let key_reference = CanonicalSecureKeyReference::parse(&request.key_id)
            .expect("canonical parser should parse secure provider key");

        let signed = deterministic_secure_provider_client_sign(&request, &key_reference)
            .expect("deterministic provider client should sign");
        assert_eq!(signed.backend, "secure-aws-kms-emulator");
    });
}
