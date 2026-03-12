use super::super::env::LOCAL_BACKEND_NAME;
use super::super::errors::SignerBackendError;
use super::super::request::SigningRequest;
use super::traits::SignerBackend;
use super::verification::{matches_legacy_signature, verify_with_expected_public_key};

/// Local deterministic software signer backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalSignerBackend;

impl SignerBackend for LocalSignerBackend {
    fn backend_name(&self) -> &'static str {
        LOCAL_BACKEND_NAME
    }

    fn sign(&self, request: &SigningRequest) -> Result<String, SignerBackendError> {
        request.expected_signature()
    }

    fn verify(&self, request: &SigningRequest, signature: &str) -> Result<(), SignerBackendError> {
        if matches_legacy_signature(request, signature) {
            return Ok(());
        }
        verify_with_expected_public_key(self.backend_name(), request, signature)
    }
}
