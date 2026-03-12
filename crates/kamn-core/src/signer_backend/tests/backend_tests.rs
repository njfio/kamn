use crate::signer_backend::backends::{SecureSignerBackend, SignerBackend};
use crate::signer_backend::provider_policy::{
    BackendSignature, CanonicalSecureKeyReference, SignerProviderHandshakeMatrix,
};
use crate::signer_backend::request::SigningRequest;
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
fn secure_backend_rejects_provider_client_backend_mismatch() {
    fn mismatched_provider_client(
        request: &SigningRequest,
        _key_reference: &CanonicalSecureKeyReference,
    ) -> Result<BackendSignature, SignerBackendError> {
        let signature = request.expected_signature()?;
        Ok(BackendSignature {
            backend: "secure-mock".to_owned(),
            signature,
        })
    }

    with_default_signer_key_env(|| {
        let backend = SecureSignerBackend::with_provider_client(
            SignerProviderHandshakeMatrix::with_uniform_availability(true),
            mismatched_provider_client,
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
            Err(SignerBackendError::ProviderClientBackendMismatch {
                expected_backend: "secure-aws-kms-emulator".to_owned(),
                provided_backend: "secure-mock".to_owned(),
                key_id: "secure:aws-kms:key-prod-1".to_owned(),
            })
        );
    });
}
