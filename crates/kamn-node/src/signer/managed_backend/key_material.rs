use std::env;

use kamn_core::ConfigError;

use super::VerifyingKey;
use super::super::{decode_kolme_hex_bytes, encode_kolme_hex_lower, KolmeLiveSignerSelection};
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
    let key_bytes = decode_kolme_hex_bytes(trimmed, env_name).map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "{env_name} is invalid: {error} (managed_signer_public_key_marker_invalid)"
        ))
    })?;
    if key_bytes.len() != 33 {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{env_name} must decode to 33 bytes compressed secp256k1 key material, found {} (managed_signer_public_key_marker_invalid)",
            key_bytes.len()
        )));
    }
    let verifying_key = VerifyingKey::from_sec1_bytes(key_bytes.as_slice()).map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "{env_name} is not valid secp256k1 key material: {error} (managed_signer_public_key_marker_invalid)"
        ))
    })?;
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
            "{env_name} must be set when --kolme-live-signer-key-source={} (managed_signer_public_key_marker_missing)",
            KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL
        ))),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{env_name} must be valid utf-8 when present (managed_signer_public_key_marker_invalid)"
        ))),
    }
}
