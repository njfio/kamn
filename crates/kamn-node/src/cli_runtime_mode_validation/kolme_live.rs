use super::super::{
    normalize_kolme_live_signer_key_source, normalize_kolme_live_signer_profile_selector,
    KOLME_IN_MEMORY_PROVIDER_MARKER, KOLME_LIVE_SIGNING_PROFILE,
};
use super::{ConfigError, RuntimeModeValidationInputs};

pub(super) fn validate_kolme_live_mode(
    inputs: &RuntimeModeValidationInputs<'_>,
) -> Result<(), ConfigError> {
    let provider_hint =
        inputs
            .kolme_live_provider_hint
            .ok_or(ConfigError::MissingArgumentValue(
                "--kolme-live-provider-hint",
            ))?;
    validate_kolme_live_required_fields(
        inputs.kolme_live_base_url,
        inputs.kolme_live_signing_profile,
    )?;
    validate_provider_hint(provider_hint)?;
    validate_signing_profile(inputs.kolme_live_signing_profile)?;
    validate_tick_pair(inputs.daemon_max_ticks, inputs.daemon_tick_interval_ms)?;
    validate_signer_key_source(inputs.kolme_live_signer_key_source)?;
    if inputs.kolme_live_strict_signer_contracts {
        validate_signer_profile(inputs.kolme_live_signer_profile)?;
    }
    Ok(())
}

fn validate_kolme_live_required_fields(
    base_url: Option<&str>,
    signing_profile: Option<&str>,
) -> Result<(), ConfigError> {
    let _ = base_url.ok_or(ConfigError::MissingArgumentValue("--kolme-live-base-url"))?;
    let _ = signing_profile.ok_or(ConfigError::MissingArgumentValue(
        "--kolme-live-signing-profile",
    ))?;
    Ok(())
}

fn validate_provider_hint(provider_hint: &str) -> Result<(), ConfigError> {
    if provider_hint.contains(KOLME_IN_MEMORY_PROVIDER_MARKER) {
        return Err(ConfigError::InvalidKolmeLiveProviderHint(
            provider_hint.to_owned(),
        ));
    }
    Ok(())
}

fn validate_signing_profile(signing_profile: Option<&str>) -> Result<(), ConfigError> {
    let signing_profile = signing_profile.ok_or(ConfigError::MissingArgumentValue(
        "--kolme-live-signing-profile",
    ))?;
    if signing_profile != KOLME_LIVE_SIGNING_PROFILE {
        return Err(ConfigError::InvalidKolmeLiveSigningProfile(
            signing_profile.to_owned(),
        ));
    }
    Ok(())
}

fn validate_tick_pair(
    daemon_max_ticks: Option<u64>,
    daemon_tick_interval_ms: Option<u64>,
) -> Result<(), ConfigError> {
    if daemon_max_ticks.is_some() && daemon_tick_interval_ms.is_none() {
        return Err(ConfigError::MissingArgumentValue(
            "--daemon-tick-interval-ms",
        ));
    }
    if daemon_tick_interval_ms.is_some() && daemon_max_ticks.is_none() {
        return Err(ConfigError::MissingArgumentValue("--daemon-max-ticks"));
    }
    Ok(())
}

fn validate_signer_key_source(signer_key_source: Option<&str>) -> Result<(), ConfigError> {
    let key_source = signer_key_source.ok_or(ConfigError::MissingArgumentValue(
        "--kolme-live-signer-key-source",
    ))?;
    normalize_kolme_live_signer_key_source(key_source)?;
    Ok(())
}

fn validate_signer_profile(signer_profile: Option<&str>) -> Result<(), ConfigError> {
    let profile = signer_profile.ok_or(ConfigError::MissingArgumentValue(
        "--kolme-live-signer-profile",
    ))?;
    normalize_kolme_live_signer_profile_selector(profile)?;
    Ok(())
}
