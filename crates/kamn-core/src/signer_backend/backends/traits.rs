use super::super::errors::SignerBackendError;
use super::super::provider_policy::{BackendSignature, CanonicalSecureKeyReference};
use super::super::request::SigningRequest;

/// Backend abstraction for signing and verification providers.
pub trait SignerBackend {
    /// Return backend identifier used in audit/verification routes.
    fn backend_name(&self) -> &'static str;
    /// Sign a validated signing request payload.
    fn sign(&self, request: &SigningRequest) -> Result<String, SignerBackendError>;
    /// Verify signature material for a signing request payload.
    fn verify(&self, request: &SigningRequest, signature: &str) -> Result<(), SignerBackendError>;
}

/// Client abstraction for provider-backed secure signing.
pub trait SecureSignerProviderClient {
    /// Produce backend-tagged signature material using provider-specific signing flow.
    fn sign_with_provider(
        &self,
        request: &SigningRequest,
        key_reference: &CanonicalSecureKeyReference,
    ) -> Result<BackendSignature, SignerBackendError>;
}
