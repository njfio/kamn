use super::super::env::{
    signer_legacy_baseline_v1_compat_enabled, resolve_signer_private_key_hex,
    LOCAL_BACKEND_NAME,
};
use super::super::errors::SignerBackendError;
use super::super::request::SigningRequest;
use super::traits::SignerBackend;
use crate::signature_profile::{
    service_auth_public_key_hex_from_private_key_hex, service_auth_verify_with_public_key_hex,
    signature_matches_supported_profile_for_fields,
};

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
                backend: self.backend_name().to_owned(),
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
