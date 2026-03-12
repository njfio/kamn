use super::provider_client::{
    deterministic_secure_provider_client_sign, SecureSignerProviderClientSignFn,
};
use super::traits::SignerBackend;
use crate::signature_profile::{
    service_auth_public_key_hex_from_private_key_hex, service_auth_verify_with_public_key_hex,
    signature_matches_supported_profile_for_fields,
};
use crate::signer_backend::env::{
    signer_legacy_baseline_v1_compat_enabled, resolve_signer_private_key_hex,
    SECURE_MOCK_BACKEND_NAME,
};
use crate::signer_backend::errors::SignerBackendError;
use crate::signer_backend::provider_policy::{
    BackendSignature, CanonicalSecureKeyReference, SecureSignerProvider, SignerKeyRole,
    SignerProviderHandshakeMatrix, SignerProviderHandshakeStatus,
};
use crate::signer_backend::request::SigningRequest;

/// Secure-provider signer backend with handshake and key-role policy enforcement.
#[derive(Debug, Clone)]
pub struct SecureSignerBackend {
    provider_handshake_matrix: SignerProviderHandshakeMatrix,
    provider_client_sign: SecureSignerProviderClientSignFn,
}

impl SecureSignerBackend {
    /// Construct secure signer backend with uniform provider availability.
    pub fn new(available: bool) -> Self {
        Self::with_provider_handshake_matrix(
            SignerProviderHandshakeMatrix::with_uniform_availability(available),
        )
    }

    /// Construct secure signer backend with explicit provider handshake matrix.
    pub fn with_provider_handshake_matrix(
        provider_handshake_matrix: SignerProviderHandshakeMatrix,
    ) -> Self {
        Self::with_provider_client(
            provider_handshake_matrix,
            deterministic_secure_provider_client_sign,
        )
    }

    /// Construct secure signer backend with explicit handshake matrix and provider client callback.
    pub fn with_provider_client(
        provider_handshake_matrix: SignerProviderHandshakeMatrix,
        provider_client_sign: SecureSignerProviderClientSignFn,
    ) -> Self {
        Self {
            provider_handshake_matrix,
            provider_client_sign,
        }
    }

    pub(crate) fn sign_with_backend(
        &self,
        request: &SigningRequest,
    ) -> Result<BackendSignature, SignerBackendError> {
        let secure_key = CanonicalSecureKeyReference::parse(&request.key_id)?;
        self.enforce_key_role_segregation(request, &secure_key)?;
        self.enforce_provider_handshake(secure_key.provider)?;
        let signed = (self.provider_client_sign)(request, &secure_key)?;
        let expected_backend = secure_key.provider.backend_name().to_owned();
        if signed.backend != expected_backend {
            return Err(SignerBackendError::ProviderClientBackendMismatch {
                expected_backend,
                provided_backend: signed.backend,
                key_id: request.key_id.clone(),
            });
        }
        Ok(signed)
    }

    pub(crate) fn verify_with_backend_name(
        &self,
        backend: &str,
        request: &SigningRequest,
        signature: &str,
    ) -> Result<(), SignerBackendError> {
        let secure_key = CanonicalSecureKeyReference::parse(&request.key_id)?;
        let expected_backend = secure_key.provider.backend_name();
        if backend != expected_backend {
            return Err(SignerBackendError::SecureProviderBackendMismatch {
                expected_backend: expected_backend.to_owned(),
                provided_backend: backend.to_owned(),
                key_id: request.key_id.clone(),
            });
        }
        self.verify(request, signature)
    }

    fn enforce_key_role_segregation(
        &self,
        request: &SigningRequest,
        secure_key: &CanonicalSecureKeyReference,
    ) -> Result<(), SignerBackendError> {
        let sender_role = SignerKeyRole::from_sender(&request.sender);
        if sender_role != secure_key.key_role {
            return Err(SignerBackendError::KeyRoleMismatch {
                key_role: secure_key.key_role.label().to_owned(),
                sender_role: sender_role.label().to_owned(),
                sender: request.sender.clone(),
                key_id: request.key_id.clone(),
            });
        }
        Ok(())
    }

    fn enforce_provider_handshake(
        &self,
        provider: SecureSignerProvider,
    ) -> Result<(), SignerBackendError> {
        let backend = provider.backend_name().to_owned();
        match self.provider_handshake_matrix.status_for_provider(provider) {
            SignerProviderHandshakeStatus::Available => Ok(()),
            SignerProviderHandshakeStatus::Unavailable => {
                Err(SignerBackendError::ProviderUnavailable { backend })
            }
            SignerProviderHandshakeStatus::PolicyBlocked => {
                Err(SignerBackendError::ProviderHandshakeRejected {
                    backend,
                    failure_class: "policy-blocked".to_owned(),
                })
            }
        }
    }
}

impl SignerBackend for SecureSignerBackend {
    fn backend_name(&self) -> &'static str {
        SECURE_MOCK_BACKEND_NAME
    }

    fn sign(&self, request: &SigningRequest) -> Result<String, SignerBackendError> {
        Ok(self.sign_with_backend(request)?.signature)
    }

    fn verify(&self, request: &SigningRequest, signature: &str) -> Result<(), SignerBackendError> {
        let secure_key = CanonicalSecureKeyReference::parse(&request.key_id)?;
        self.enforce_key_role_segregation(request, &secure_key)?;
        self.enforce_provider_handshake(secure_key.provider)?;
        if signer_legacy_baseline_v1_compat_enabled()
            && signature_matches_supported_profile_for_fields(
                signature,
                &request.sender,
                request.nonce,
                &request.state_hash,
                &request.payload,
            )
        {
            return Ok(());
        }
        let expected = request.expected_signature()?;
        let expected_public_key_hex = expected_public_key_hex(request)?;
        if service_auth_verify_with_public_key_hex(
            signature,
            &request.sender,
            request.nonce,
            &request.state_hash,
            &request.payload,
            expected_public_key_hex.as_str(),
        )
        .is_err()
        {
            return Err(SignerBackendError::SignatureMismatch {
                backend: secure_key.provider.backend_name().to_owned(),
                expected,
                found: signature.to_owned(),
            });
        }
        Ok(())
    }
}

fn expected_public_key_hex(request: &SigningRequest) -> Result<String, SignerBackendError> {
    let private_key_hex = resolve_signer_private_key_hex(request.key_id.as_str())?;
    service_auth_public_key_hex_from_private_key_hex(private_key_hex.as_str()).map_err(|_| {
        SignerBackendError::InvalidSigningKeyMaterial {
            key_id: request.key_id.clone(),
        }
    })
}
