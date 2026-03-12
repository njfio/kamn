use super::models::{KolmeLiveSignerSecretProvider, KolmeLiveSignerSelection};
use kamn_core::ConfigError;
use crate::{
    KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL, KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL,
    KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_ENV, KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV,
    KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY_ENV, KOLME_LIVE_SIGNER_PROFILE_PRIMARY,
    KOLME_LIVE_SIGNER_PROFILE_SECONDARY,
};
use zeroize::Zeroize;
use std::env;

pub(crate) struct EnvKolmeLiveSignerSecretProvider;

impl KolmeLiveSignerSecretProvider for EnvKolmeLiveSignerSecretProvider {
    fn ensure_no_fallback_private_key_path(&self) -> Result<(), ConfigError> {
        match env::var(KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV) {
            Ok(_) => Err(ConfigError::RuntimeKolmeLive(format!(
                "{KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV} must remain unset (fallback_signer_secret_present_violation)"
            ))),
            Err(env::VarError::NotPresent) => Ok(()),
            Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
                "{KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV} must be valid utf-8 when present (fallback_signer_secret_present_violation)"
            ))),
        }
    }

    fn read_private_key_hex(
        &self,
        selection: &KolmeLiveSignerSelection,
    ) -> Result<String, ConfigError> {
        match env::var(selection.private_key_env) {
            Ok(private_key_hex) => Ok(private_key_hex),
            Err(env::VarError::NotPresent) => Err(ConfigError::RuntimeKolmeLive(format!(
                "{} must be set for signer profile {}",
                selection.private_key_env, selection.profile
            ))),
            Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
                "{} must be valid utf-8 for signer profile {}",
                selection.private_key_env, selection.profile
            ))),
        }
    }
}

pub(crate) fn ensure_kolme_live_strict_signer_secret_source_precedence(
    strict_signer_profile: Option<&str>,
    selection: &KolmeLiveSignerSelection,
) -> Result<(), ConfigError> {
    if strict_signer_profile.is_none()
        || selection.key_source != KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL
    {
        return Ok(());
    }
    let non_selected_private_key_env = match selection.profile {
        KOLME_LIVE_SIGNER_PROFILE_PRIMARY => KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY_ENV,
        KOLME_LIVE_SIGNER_PROFILE_SECONDARY => KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_ENV,
        _ => {
            return Err(ConfigError::RuntimeKolmeLive(format!(
                "unsupported signer profile for strict secret-source precedence checks: {} (signer_secret_source_precedence_violation)",
                selection.profile
            )))
        }
    };
    match env::var(non_selected_private_key_env) {
        Ok(_) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{non_selected_private_key_env} must remain unset when --kolme-live-signer-profile={} and --kolme-live-signer-key-source={} select {} (signer_secret_source_precedence_violation)",
            selection.profile, selection.key_source, selection.private_key_env
        ))),
        Err(env::VarError::NotPresent) => Ok(()),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{non_selected_private_key_env} must be valid utf-8 when present under strict signer source contracts (signer_secret_source_precedence_violation)"
        ))),
    }
}

pub(crate) fn ensure_kolme_live_strict_signer_secret_source_precedence_and_zeroize(
    strict_signer_profile: Option<&str>,
    selection: &KolmeLiveSignerSelection,
    private_key_hex: &mut String,
) -> Result<(), ConfigError> {
    if let Err(error) =
        ensure_kolme_live_strict_signer_secret_source_precedence(strict_signer_profile, selection)
    {
        private_key_hex.zeroize();
        return Err(error);
    }
    Ok(())
}

pub(crate) fn ensure_kolme_live_managed_external_private_key_env_unset(
    selection: &KolmeLiveSignerSelection,
) -> Result<(), ConfigError> {
    match env::var(selection.private_key_env) {
        Ok(_) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{} must remain unset for signer profile {} when --kolme-live-signer-key-source={} (managed_signer_raw_private_key_forbidden)",
            selection.private_key_env, selection.profile, KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL
        ))),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{} must be valid utf-8 when present for signer profile {} (managed_signer_raw_private_key_forbidden)",
            selection.private_key_env, selection.profile
        ))),
        Err(env::VarError::NotPresent) => Ok(()),
    }
}
