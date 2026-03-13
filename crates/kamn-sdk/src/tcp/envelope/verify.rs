use super::super::support::map_from_did_key_binding_error;
use crate::{AgentDid, SdkError};
use kamn_core::{
    service_auth_public_key_hex_from_private_key_hex, service_auth_sign_with_private_key_hex,
    ServiceAuthSignatureError,
};

pub(super) fn verify_state_hash_shape(state_hash: &str) -> Result<(), SdkError> {
    verify_non_empty_single_line(state_hash, "state_hash")
}

pub(super) fn verify_body_shape(body: &str) -> Result<(), SdkError> {
    verify_non_empty_single_line(body, "body")
}

pub(super) fn verify_signer_public_key_shape(signer_public_key: &str) -> Result<(), SdkError> {
    verify_non_empty_single_line(signer_public_key, "signer_public_key")
}

pub(super) fn map_signature_verify_error(error: ServiceAuthSignatureError) -> SdkError {
    match error {
        ServiceAuthSignatureError::InvalidPublicKeyHex
        | ServiceAuthSignatureError::EmptyField("expected_public_key_hex") => {
            SdkError::InvalidInput {
                field: "signer_public_key",
                reason: "must be valid compressed secp256k1 public key hex",
            }
        }
        _ => SdkError::InvalidInput {
            field: "signature",
            reason: "failed cryptographic envelope verification",
        },
    }
}

pub(super) fn verify_did_key_binding(
    from: &AgentDid,
    signer_public_key: &str,
) -> Result<(), SdkError> {
    from.ensure_public_key_hex_binding(signer_public_key)
        .map_err(map_from_did_key_binding_error)
}

pub(super) fn derive_signer_public_key(signer_private_key_hex: &str) -> Result<String, SdkError> {
    service_auth_public_key_hex_from_private_key_hex(signer_private_key_hex).map_err(|_| {
        SdkError::InvalidInput {
            field: "signer_private_key",
            reason: "must be valid secp256k1 private key hex",
        }
    })
}

pub(super) fn sign_envelope_fields(
    from: &str,
    nonce: u64,
    state_hash: &str,
    body: &str,
    signer_private_key_hex: &str,
) -> Result<String, SdkError> {
    service_auth_sign_with_private_key_hex(from, nonce, state_hash, body, signer_private_key_hex)
        .map_err(|_| SdkError::InvalidInput {
            field: "signer_private_key",
            reason: "failed to sign tcp envelope fields",
        })
}

fn verify_non_empty_single_line(value: &str, field: &'static str) -> Result<(), SdkError> {
    if value.trim().is_empty() {
        return Err(SdkError::InvalidInput {
            field,
            reason: "must not be empty",
        });
    }
    if value.contains('\n') || value.contains('\r') {
        return Err(SdkError::InvalidInput {
            field,
            reason: "must be single-line",
        });
    }
    Ok(())
}
