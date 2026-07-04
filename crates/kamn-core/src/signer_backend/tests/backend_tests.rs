use crate::signer_backend::backends::{SecureSignerBackend, SignerBackend};
use crate::signer_backend::provider_policy::{
    BackendSignature, CanonicalSecureKeyReference, SignerProviderHandshakeMatrix,
};
use crate::signer_backend::request::SigningRequest;
use crate::signer_backend::tests::support::with_default_signer_key_env;
use crate::signer_backend::SignerBackendError;

#[test]
fn secure_backend_rejects_provider_client_backend_mismatch() {
    with_default_signer_key_env(|| {
        assert_eq!(
            build_backend().sign(&build_request()),
            Err(expected_mismatch_error())
        );
    });
}

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

fn build_backend() -> SecureSignerBackend {
    SecureSignerBackend::with_provider_client(
        SignerProviderHandshakeMatrix::with_uniform_availability(true),
        mismatched_provider_client,
    )
}

fn build_request() -> SigningRequest {
    SigningRequest::new(
        "secure:aws-kms:key-prod-1",
        "agent-a",
        1,
        "payload-1",
        "state:genesis",
    )
    .expect("request should be valid")
}

fn expected_mismatch_error() -> SignerBackendError {
    SignerBackendError::ProviderClientBackendMismatch {
        expected_backend: "secure-aws-kms-emulator".to_owned(),
        provided_backend: "secure-mock".to_owned(),
        key_id: "secure:aws-kms:key-prod-1".to_owned(),
    }
}
