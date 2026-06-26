use super::errors::SignerBackendError;
use crate::SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV;
use std::env;

pub(super) const LOCAL_BACKEND_NAME: &str = "local-software";
pub(super) const SECURE_MOCK_BACKEND_NAME: &str = "secure-mock";
pub(super) const SECURE_AWS_KMS_BACKEND_NAME: &str = "secure-aws-kms-emulator";
pub(super) const SIGNER_PRIVATE_KEY_ENV: &str = "KAMN_SIGNER_PRIVATE_KEY_HEX";
pub(super) const SIGNER_PRIVATE_KEY_ENV_PREFIX: &str = "KAMN_SIGNER_PRIVATE_KEY_HEX__";
const SIGNER_LEGACY_BASELINE_V1_COMPAT_ENV: &str = "KAMN_SIGNER_ALLOW_LEGACY_BASELINE_V1";

pub(super) fn signer_key_id_private_key_env_name(key_id: &str) -> String {
    let mut normalized = String::with_capacity(key_id.len());
    for byte in key_id.bytes() {
        if byte.is_ascii_alphanumeric() {
            normalized.push((byte as char).to_ascii_uppercase());
        } else {
            normalized.push('_');
        }
    }
    format!("{SIGNER_PRIVATE_KEY_ENV_PREFIX}{normalized}")
}

pub(super) fn signer_legacy_baseline_v1_compat_enabled_for_mode_with_env_value(
    debug_assertions: bool,
    env_value: Option<&str>,
) -> bool {
    if !debug_assertions {
        return false;
    }
    match env_value {
        Some(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        None => false,
    }
}

pub(super) fn signer_legacy_baseline_v1_compat_enabled_for_mode(debug_assertions: bool) -> bool {
    let env_value = env::var(SIGNER_LEGACY_BASELINE_V1_COMPAT_ENV).ok();
    signer_legacy_baseline_v1_compat_enabled_for_mode_with_env_value(
        debug_assertions,
        env_value.as_deref(),
    )
}

pub(super) fn signer_legacy_baseline_v1_compat_enabled() -> bool {
    signer_legacy_baseline_v1_compat_enabled_for_mode(cfg!(debug_assertions))
}

pub(super) fn resolve_signer_private_key_hex(key_id: &str) -> Result<String, SignerBackendError> {
    let key_specific_env = signer_key_id_private_key_env_name(key_id);
    if let Some(value) = non_empty_env_value(key_specific_env.as_str()) {
        return Ok(value);
    }
    if let Some(value) = first_fallback_signing_key() {
        return Ok(value);
    }
    Err(SignerBackendError::MissingSigningKeyMaterial {
        key_id: key_id.to_owned(),
        key_specific_env,
    })
}

fn first_fallback_signing_key() -> Option<String> {
    [
        SIGNER_PRIVATE_KEY_ENV,
        SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV,
    ]
    .into_iter()
    .find_map(non_empty_env_value)
}

fn non_empty_env_value(env_name: &str) -> Option<String> {
    env::var(env_name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}
