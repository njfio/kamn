mod decoding;
mod verification;

use kamn_core::ConfigError;

use super::{ManagedExternalBackendSignature, RecoveryId, VerifyingKey};
use super::super::encode_kolme_hex_lower;
use decoding::{
    decode_signer_verifying_key, missing_recovery_id, missing_signature_hex, missing_signer_key,
    parse_recovery_id, require_signer_public_key, require_value, validate_signature_material,
};
use verification::{verify_public_key_match, verify_signature_matches_message};

pub(super) fn parse_kolme_live_managed_signer_command_output(
    stdout: &str,
) -> Result<ManagedExternalBackendSignature, ConfigError> {
    let mut signature_hex = None;
    let mut recovery_id = None;
    let mut signer_public_key_hex = None;
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (key, value) = trimmed.split_once('=').ok_or_else(|| {
            ConfigError::RuntimeKolmeLive(format!(
                "managed-external signer backend response line must be key=value, found '{trimmed}' (managed_signer_backend_response_malformed)"
            ))
        })?;
        match key.trim() {
            "signature_hex" => signature_hex = Some(require_value(value, "signature_hex")?),
            "recovery_id" => recovery_id = Some(parse_recovery_id(value)?),
            "signer_public_key_hex" => {
                signer_public_key_hex = Some(require_signer_public_key(value)?)
            }
            _ => {}
        }
    }
    let signature_hex = signature_hex.ok_or_else(missing_signature_hex)?;
    let recovery_id = recovery_id.ok_or_else(missing_recovery_id)?;
    validate_signature_material(signature_hex.as_str(), recovery_id)?;
    let signer_public_key_hex = signer_public_key_hex.ok_or_else(missing_signer_key)?;
    let signer_verifying_key = decode_signer_verifying_key(signer_public_key_hex.as_str())?;
    Ok(ManagedExternalBackendSignature {
        signature_hex,
        recovery_id,
        signer_public_key_hex: encode_kolme_hex_lower(
            signer_verifying_key.to_encoded_point(true).as_bytes(),
        ),
    })
}

pub(super) fn verify_kolme_live_managed_signer_backend_signature_provenance(
    canonical_message: &str,
    expected_signer_public_key_hex: &str,
    backend_signature: &ManagedExternalBackendSignature,
) -> Result<(), ConfigError> {
    let expected_signer_public_key_hex = expected_signer_public_key_hex.trim();
    if expected_signer_public_key_hex.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(
            "expected managed-external signer public key must not be empty (managed_signer_backend_response_provenance_mismatch)"
                .to_owned(),
        ));
    }
    verify_public_key_match(expected_signer_public_key_hex, backend_signature)?;
    verify_signature_matches_message(canonical_message, backend_signature)
}
