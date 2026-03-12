use std::env;

use kamn_core::ConfigError;

use super::parsing::parse_kolme_live_managed_signer_command_spec;
use crate::{
    KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV, KOLME_LIVE_MANAGED_SIGNER_REQUIRED_ENV,
    KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_DEFAULT,
    KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_ENV,
};

fn resolve_optional_kolme_live_managed_signer_command() -> Result<Option<String>, ConfigError> {
    match env::var(KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV) {
        Ok(command) => validate_optional_command(command),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV} must be valid utf-8 when present (managed_signer_backend_unavailable)"
        ))),
    }
}

fn validate_optional_command(command: String) -> Result<Option<String>, ConfigError> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV} must not be empty when present (managed_signer_backend_unavailable)"
        )));
    }
    parse_kolme_live_managed_signer_command_spec(trimmed)?;
    Ok(Some(trimmed.to_owned()))
}

fn parse_positive_timeout(trimmed: &str) -> Result<u64, ConfigError> {
    let timeout = trimmed.parse::<u64>().map_err(|_| {
        ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_ENV} must be a positive integer, found '{trimmed}' (managed_signer_backend_timeout_invalid)"
        ))
    })?;
    if timeout == 0 {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_ENV} must be greater than zero (managed_signer_backend_timeout_invalid)"
        )));
    }
    Ok(timeout)
}

fn parse_required_marker_value(trimmed: &str) -> Result<bool, ConfigError> {
    match trimmed {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_MANAGED_SIGNER_REQUIRED_ENV} must be 'true' or 'false', found '{trimmed}' (managed_signer_backend_required_invalid)"
        ))),
    }
}

pub(crate) fn resolve_required_kolme_live_managed_signer_command() -> Result<String, ConfigError> {
    resolve_optional_kolme_live_managed_signer_command()?.ok_or_else(|| {
        ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV} must be set when managed-external signing is selected (managed_signer_backend_required_missing)"
        ))
    })
}

pub(crate) fn resolve_kolme_live_managed_signer_required_marker() -> Result<bool, ConfigError> {
    match env::var(KOLME_LIVE_MANAGED_SIGNER_REQUIRED_ENV) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "{KOLME_LIVE_MANAGED_SIGNER_REQUIRED_ENV} must not be empty when present (managed_signer_backend_required_invalid)"
                )));
            }
            parse_required_marker_value(trimmed)
        }
        Err(env::VarError::NotPresent) => Ok(false),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_MANAGED_SIGNER_REQUIRED_ENV} must be valid utf-8 when present (managed_signer_backend_required_invalid)"
        ))),
    }
}

pub(crate) fn resolve_kolme_live_managed_signer_timeout_seconds() -> Result<u64, ConfigError> {
    match env::var(KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_ENV) {
        Ok(raw_timeout) => {
            let trimmed = raw_timeout.trim();
            if trimmed.is_empty() {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "{KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_ENV} must not be empty when present (managed_signer_backend_timeout_invalid)"
                )));
            }
            parse_positive_timeout(trimmed)
        }
        Err(env::VarError::NotPresent) => Ok(KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_DEFAULT),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_ENV} must be valid utf-8 when present (managed_signer_backend_timeout_invalid)"
        ))),
    }
}
