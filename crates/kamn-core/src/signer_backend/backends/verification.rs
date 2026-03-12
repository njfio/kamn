use crate::signature_profile::{
    service_auth_public_key_hex_from_private_key_hex, service_auth_verify_with_public_key_hex,
    signature_matches_supported_profile_for_fields,
};
use crate::signer_backend::env::{
    resolve_signer_private_key_hex, signer_legacy_baseline_v1_compat_enabled,
};
use crate::signer_backend::errors::SignerBackendError;
use crate::signer_backend::request::SigningRequest;

pub(super) fn matches_legacy_signature(request: &SigningRequest, signature: &str) -> bool {
    signer_legacy_baseline_v1_compat_enabled()
        && signature_matches_supported_profile_for_fields(
            signature,
            &request.sender,
            request.nonce,
            &request.state_hash,
            &request.payload,
        )
}

pub(super) fn verify_with_expected_public_key(
    backend_name: &str,
    request: &SigningRequest,
    signature: &str,
) -> Result<(), SignerBackendError> {
    let expected = request.expected_signature()?;
    let expected_public_key_hex = expected_public_key_hex(request)?;
    if signature_matches_expected_public_key(request, signature, &expected_public_key_hex) {
        return Ok(());
    }
    Err(SignerBackendError::SignatureMismatch {
        backend: backend_name.to_owned(),
        expected,
        found: signature.to_owned(),
    })
}

fn signature_matches_expected_public_key(
    request: &SigningRequest,
    signature: &str,
    expected_public_key_hex: &str,
) -> bool {
    service_auth_verify_with_public_key_hex(
        signature,
        &request.sender,
        request.nonce,
        &request.state_hash,
        &request.payload,
        expected_public_key_hex,
    )
    .is_ok()
}

fn expected_public_key_hex(request: &SigningRequest) -> Result<String, SignerBackendError> {
    let private_key_hex = resolve_signer_private_key_hex(request.key_id.as_str())?;
    service_auth_public_key_hex_from_private_key_hex(private_key_hex.as_str()).map_err(|_| {
        SignerBackendError::InvalidSigningKeyMaterial {
            key_id: request.key_id.clone(),
        }
    })
}
