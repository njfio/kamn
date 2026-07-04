use super::backends::{
    LocalSignerBackend, SecureSignerBackend, SecureSignerProviderClientSignFn, SignerBackend,
};
use super::env::{LOCAL_BACKEND_NAME, SECURE_AWS_KMS_BACKEND_NAME, SECURE_MOCK_BACKEND_NAME};
use super::errors::SignerBackendError;
use super::provider_policy::{SignerKeyRole, SignerProviderHandshakeMatrix};
use super::request::SigningRequest;
use super::signing_cache::SignerBackendSigningCache;
use std::sync::Arc;

/// Router that prefers secure signer paths and applies policy fallback to local backend.
#[derive(Debug, Clone)]
pub struct SignerBackendRouter {
    local: LocalSignerBackend,
    secure: SecureSignerBackend,
    signing_cache: Arc<SignerBackendSigningCache>,
}

impl SignerBackendRouter {
    /// Construct router using uniform secure-provider availability.
    pub fn with_secure_availability(secure_available: bool) -> Self {
        Self::with_provider_handshake_matrix(
            SignerProviderHandshakeMatrix::with_uniform_availability(secure_available),
        )
    }

    /// Construct router with explicit provider handshake matrix.
    pub fn with_provider_handshake_matrix(
        provider_handshake_matrix: SignerProviderHandshakeMatrix,
    ) -> Self {
        let signing_cache = Arc::new(SignerBackendSigningCache::default());
        Self {
            local: LocalSignerBackend,
            secure: SecureSignerBackend::with_deterministic_provider_client_and_cache(
                provider_handshake_matrix,
                Arc::clone(&signing_cache),
            ),
            signing_cache,
        }
    }

    /// Construct router with explicit provider handshake matrix and provider-client callback.
    pub fn with_provider_client(
        provider_handshake_matrix: SignerProviderHandshakeMatrix,
        provider_client_sign: SecureSignerProviderClientSignFn,
    ) -> Self {
        let signing_cache = Arc::new(SignerBackendSigningCache::default());
        Self {
            local: LocalSignerBackend,
            secure: SecureSignerBackend::with_provider_client_and_cache(
                provider_handshake_matrix,
                provider_client_sign,
                Arc::clone(&signing_cache),
            ),
            signing_cache,
        }
    }

    /// Sign request with secure backend first, falling back to local backend when policy permits.
    pub fn sign_with_secure_fallback(
        &self,
        request: &SigningRequest,
    ) -> Result<super::provider_policy::BackendSignature, SignerBackendError> {
        match self.secure.sign_with_backend(request) {
            Ok(signature) => Ok(signature),
            Err(SignerBackendError::ProviderUnavailable { .. }) => {
                let key_role = SignerKeyRole::from_key_id(&request.key_id)?;
                if !key_role.allows_secure_fallback() {
                    return Err(SignerBackendError::FallbackDeniedByRolePolicy {
                        key_role: key_role.label().to_owned(),
                        key_id: request.key_id.clone(),
                    });
                }
                let signature = self
                    .local
                    .sign_with_cache(request, self.signing_cache.as_ref())?;
                Ok(super::provider_policy::BackendSignature {
                    backend: self.local.backend_name().to_owned(),
                    signature,
                })
            }
            Err(error) => Err(error),
        }
    }

    /// Verify request signature against an explicit backend identity.
    pub fn verify_with_backend(
        &self,
        backend: &str,
        request: &SigningRequest,
        signature: &str,
    ) -> Result<(), SignerBackendError> {
        match backend {
            LOCAL_BACKEND_NAME => self.local.verify(request, signature),
            SECURE_MOCK_BACKEND_NAME | SECURE_AWS_KMS_BACKEND_NAME => self
                .secure
                .verify_with_backend_name(backend, request, signature),
            _ => Err(SignerBackendError::UnknownBackend(backend.to_owned())),
        }
    }
}

impl Default for SignerBackendRouter {
    fn default() -> Self {
        Self::with_secure_availability(true)
    }
}
