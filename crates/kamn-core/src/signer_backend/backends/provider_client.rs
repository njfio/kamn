use super::super::errors::SignerBackendError;
use super::super::provider_policy::{BackendSignature, CanonicalSecureKeyReference};
use super::super::request::SigningRequest;
use super::traits::SecureSignerProviderClient;
use std::fmt;

/// Deterministic provider client used by default secure backend wiring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeterministicSecureSignerProviderClient;

impl SecureSignerProviderClient for DeterministicSecureSignerProviderClient {
    fn sign_with_provider(
        &self,
        request: &SigningRequest,
        key_reference: &CanonicalSecureKeyReference,
    ) -> Result<BackendSignature, SignerBackendError> {
        let signature = request.expected_signature()?;
        Ok(BackendSignature {
            backend: key_reference.provider.backend_name().to_owned(),
            signature,
        })
    }
}

/// Function pointer contract for secure-provider client signing callbacks.
pub type SecureSignerProviderClientSignFn = fn(
    request: &SigningRequest,
    key_reference: &CanonicalSecureKeyReference,
) -> Result<BackendSignature, SignerBackendError>;

/// Produce a deterministic backend-tagged signature for a secure provider request.
pub fn deterministic_secure_provider_client_sign(
    request: &SigningRequest,
    key_reference: &CanonicalSecureKeyReference,
) -> Result<BackendSignature, SignerBackendError> {
    DeterministicSecureSignerProviderClient.sign_with_provider(request, key_reference)
}

#[derive(Clone, Copy)]
pub(super) enum SecureSignerProviderClientMode {
    Deterministic,
    Custom(SecureSignerProviderClientSignFn),
}

impl fmt::Debug for SecureSignerProviderClientMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deterministic => formatter.write_str("Deterministic"),
            Self::Custom(_) => formatter.write_str("Custom(<fn>)"),
        }
    }
}
