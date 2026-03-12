use kamn_core::ConfigError;

use super::super::{ManagedExternalBackendSignature, RecoveryId, Signature, VerifyingKey};
use super::decoding::{decode_signature_bytes, decode_signer_verifying_key};

pub(super) fn verify_public_key_match(
    expected_signer_public_key_hex: &str,
    backend_signature: &ManagedExternalBackendSignature,
) -> Result<(), ConfigError> {
    let expected = ascii_lowercase_bytes(expected_signer_public_key_hex.as_bytes());
    let actual = ascii_lowercase_bytes(backend_signature.signer_public_key_hex.as_bytes());
    if constant_time_eq_bytes(actual.as_slice(), expected.as_slice()) {
        return Ok(());
    }
    Err(ConfigError::RuntimeKolmeLive(format!(
        "managed-external signer backend response signer_public_key_hex does not match expected runtime signer key material (expected={}, found={}) (managed_signer_backend_response_provenance_mismatch)",
        expected_signer_public_key_hex,
        backend_signature.signer_public_key_hex,
    )))
}

pub(super) fn verify_signature_matches_message(
    canonical_message: &str,
    backend_signature: &ManagedExternalBackendSignature,
) -> Result<(), ConfigError> {
    let signature_bytes = decode_signature_bytes(backend_signature.signature_hex.as_str())?;
    let signature = Signature::from_slice(signature_bytes.as_slice()).map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend response signature bytes are invalid secp256k1 material: {error} (managed_signer_backend_response_malformed)"
        ))
    })?;
    let recovery = RecoveryId::from_byte(backend_signature.recovery_id).ok_or_else(|| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend response recovery_id must be within secp256k1 range [0,3], found {} (managed_signer_backend_response_malformed)",
            backend_signature.recovery_id
        ))
    })?;
    let recovered = VerifyingKey::recover_from_msg(canonical_message.as_bytes(), &signature, recovery)
        .map_err(|error| {
            ConfigError::RuntimeKolmeLive(format!(
                "failed to recover secp256k1 public key from managed-external signer backend response: {error} (managed_signer_backend_response_malformed)"
            ))
        })?;
    let expected = decode_signer_verifying_key(backend_signature.signer_public_key_hex.as_str())?;
    if recovered == expected {
        return Ok(());
    }
    Err(ConfigError::RuntimeKolmeLive(
        "managed-external signer backend response signature does not match signer_public_key_hex (managed_signer_backend_response_provenance_mismatch)"
            .to_owned(),
    ))
}

fn ascii_lowercase_bytes(value: &[u8]) -> Vec<u8> {
    value.iter().map(u8::to_ascii_lowercase).collect()
}

fn constant_time_eq_bytes(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0_u8;
    for (lhs, rhs) in left.iter().zip(right.iter()) {
        diff |= lhs ^ rhs;
    }
    diff == 0
}
