use std::env;

use kamn_core::ConfigError;

use super::super::{decode_kolme_hex_bytes, encode_kolme_hex_lower, KolmeLiveSignerSelection};
use super::VerifyingKey;
use crate::{
    KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL, KOLME_LIVE_SIGNER_PROFILE_PRIMARY,
    KOLME_LIVE_SIGNER_PROFILE_SECONDARY, KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_ENV,
    KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY_ENV,
};

fn managed_signer_public_key_env_for_profile(profile: &str) -> Result<&'static str, ConfigError> {
    match profile {
        KOLME_LIVE_SIGNER_PROFILE_PRIMARY => Ok(KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_ENV),
        KOLME_LIVE_SIGNER_PROFILE_SECONDARY => Ok(KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY_ENV),
        _ => Err(ConfigError::RuntimeKolmeLive(format!(
            "unsupported managed-external signer profile for public key marker resolution: {profile} (managed_signer_public_key_marker_invalid)"
        ))),
    }
}

fn decode_managed_signer_public_key_bytes(
    value: &str,
    env_name: &str,
) -> Result<Vec<u8>, ConfigError> {
    let key_bytes = decode_kolme_hex_bytes(value, env_name).map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "{env_name} is invalid: {error} (managed_signer_public_key_marker_invalid)"
        ))
    })?;
    if key_bytes.len() == 33 {
        return Ok(key_bytes);
    }
    let key_len = key_bytes.len();
    Err(ConfigError::RuntimeKolmeLive(format!(
        "{env_name} must decode to 33 bytes compressed secp256k1 key material, found {key_len} (managed_signer_public_key_marker_invalid)"
    )))
}

fn parse_managed_signer_verifying_key(
    key_bytes: &[u8],
    env_name: &str,
) -> Result<VerifyingKey, ConfigError> {
    VerifyingKey::from_sec1_bytes(key_bytes).map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "{env_name} is not valid secp256k1 key material: {error} (managed_signer_public_key_marker_invalid)"
        ))
    })
}

fn normalize_managed_signer_public_key_hex(
    value: &str,
    env_name: &str,
) -> Result<String, ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{env_name} must not be empty (managed_signer_public_key_marker_invalid)"
        )));
    }
    let key_bytes = decode_managed_signer_public_key_bytes(trimmed, env_name)?;
    let verifying_key = parse_managed_signer_verifying_key(key_bytes.as_slice(), env_name)?;
    Ok(encode_kolme_hex_lower(
        verifying_key.to_encoded_point(true).as_bytes(),
    ))
}

pub(crate) fn resolve_required_managed_signer_public_key_hex(
    signer_selection: &KolmeLiveSignerSelection,
) -> Result<String, ConfigError> {
    let env_name = managed_signer_public_key_env_for_profile(signer_selection.profile)?;
    match env::var(env_name) {
        Ok(value) => normalize_managed_signer_public_key_hex(value.as_str(), env_name),
        Err(env::VarError::NotPresent) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{env_name} must be set when --kolme-live-signer-key-source={KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL} (managed_signer_public_key_marker_missing)"
        ))),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{env_name} must be valid utf-8 when present (managed_signer_public_key_marker_invalid)"
        ))),
    }
}
