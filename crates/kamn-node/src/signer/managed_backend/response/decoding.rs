use kamn_core::ConfigError;

use super::super::super::decode_kolme_hex_bytes;
use super::{RecoveryId, VerifyingKey};

pub(super) fn require_value(value: &str, key: &str) -> Result<String, ConfigError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend response missing {key} value (managed_signer_backend_response_malformed)"
        )));
    }
    Ok(value.to_owned())
}

pub(super) fn parse_recovery_id(value: &str) -> Result<u8, ConfigError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(
            "managed-external signer backend response missing recovery_id value (managed_signer_backend_response_malformed)"
                .to_owned(),
        ));
    }
    value.parse::<u8>().map_err(|_| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend response recovery_id must be an integer, found '{value}' (managed_signer_backend_response_malformed)"
        ))
    })
}

pub(super) fn require_signer_public_key(value: &str) -> Result<String, ConfigError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(
            "managed-external signer backend response missing signer_public_key_hex value (managed_signer_backend_response_provenance_missing)"
                .to_owned(),
        ));
    }
    Ok(value.to_owned())
}

pub(super) fn missing_signature_hex() -> ConfigError {
    ConfigError::RuntimeKolmeLive(
        "managed-external signer backend response missing signature_hex key (managed_signer_backend_response_malformed)"
            .to_owned(),
    )
}

pub(super) fn missing_recovery_id() -> ConfigError {
    ConfigError::RuntimeKolmeLive(
        "managed-external signer backend response missing recovery_id key (managed_signer_backend_response_malformed)"
            .to_owned(),
    )
}

pub(super) fn missing_signer_key() -> ConfigError {
    ConfigError::RuntimeKolmeLive(
        "managed-external signer backend response missing signer_public_key_hex key (managed_signer_backend_response_provenance_missing)"
            .to_owned(),
    )
}

pub(super) fn validate_signature_material(
    signature_hex: &str,
    recovery_id: u8,
) -> Result<(), ConfigError> {
    let signature_bytes = decode_signature_bytes(signature_hex)?;
    if signature_bytes.len() != 64 {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend response signature hex must decode to 64 bytes, found {} (managed_signer_backend_response_malformed)",
            signature_bytes.len()
        )));
    }
    if RecoveryId::from_byte(recovery_id).is_none() {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend response recovery_id must be within secp256k1 range [0,3], found {recovery_id} (managed_signer_backend_response_malformed)"
        )));
    }
    Ok(())
}

pub(super) fn decode_signer_verifying_key(
    public_key_hex: &str,
) -> Result<VerifyingKey, ConfigError> {
    let signer_public_key_bytes = decode_kolme_hex_bytes(
        public_key_hex,
        "managed_external_signer_backend_signer_public_key_hex",
    )
    .map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend response signer_public_key_hex is invalid: {error} (managed_signer_backend_response_provenance_malformed)"
        ))
    })?;
    if signer_public_key_bytes.len() != 33 {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend response signer_public_key_hex must decode to 33 bytes compressed secp256k1 key material, found {} (managed_signer_backend_response_provenance_malformed)",
            signer_public_key_bytes.len()
        )));
    }
    VerifyingKey::from_sec1_bytes(signer_public_key_bytes.as_slice()).map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend response signer_public_key_hex is not valid secp256k1 key material: {error} (managed_signer_backend_response_provenance_malformed)"
        ))
    })
}

pub(super) fn decode_signature_bytes(signature_hex: &str) -> Result<Vec<u8>, ConfigError> {
    decode_kolme_hex_bytes(
        signature_hex,
        "managed_external_signer_backend_signature_hex",
    )
    .map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend response signature hex is invalid: {error} (managed_signer_backend_response_malformed)"
        ))
    })
}
