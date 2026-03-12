use std::env;

use kamn_core::ConfigError;

use super::ManagedSignerCommandSpec;
use crate::{
    KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV, KOLME_LIVE_MANAGED_SIGNER_REQUIRED_ENV,
    KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_DEFAULT,
    KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_ENV,
};

fn resolve_optional_kolme_live_managed_signer_command() -> Result<Option<String>, ConfigError> {
    match env::var(KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV) {
        Ok(command) => {
            let trimmed = command.trim();
            if trimmed.is_empty() {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "{KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV} must not be empty when present (managed_signer_backend_unavailable)"
                )));
            }
            parse_kolme_live_managed_signer_command_spec(trimmed)?;
            Ok(Some(trimmed.to_owned()))
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV} must be valid utf-8 when present (managed_signer_backend_unavailable)"
        ))),
    }
}

pub(super) fn parse_kolme_live_managed_signer_command_spec(
    command: &str,
) -> Result<ManagedSignerCommandSpec, ConfigError> {
    let mut argv = Vec::new();
    let mut current = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut escaping = false;

    for character in command.chars() {
        if escaping {
            current.push(character);
            escaping = false;
            continue;
        }
        if in_single_quotes {
            if character == '\'' {
                in_single_quotes = false;
            } else {
                current.push(character);
            }
            continue;
        }
        if in_double_quotes {
            match character {
                '"' => in_double_quotes = false,
                '\\' => escaping = true,
                _ => current.push(character),
            }
            continue;
        }
        match character {
            '\'' => in_single_quotes = true,
            '"' => in_double_quotes = true,
            '\\' => escaping = true,
            character if character.is_whitespace() => {
                if !current.is_empty() {
                    argv.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(character),
        }
    }

    if escaping || in_single_quotes || in_double_quotes {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV} contains unterminated quoting or escaping (managed_signer_backend_unavailable)"
        )));
    }
    if !current.is_empty() {
        argv.push(current);
    }
    if argv.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV} must contain at least one argv token (managed_signer_backend_unavailable)"
        )));
    }
    let executable = argv.remove(0);
    Ok(ManagedSignerCommandSpec {
        executable,
        args: argv,
    })
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
            match trimmed {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(ConfigError::RuntimeKolmeLive(format!(
                    "{KOLME_LIVE_MANAGED_SIGNER_REQUIRED_ENV} must be 'true' or 'false', found '{trimmed}' (managed_signer_backend_required_invalid)"
                ))),
            }
        }
        Err(env::VarError::NotPresent) => Ok(false),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_MANAGED_SIGNER_REQUIRED_ENV} must be valid utf-8 when present (managed_signer_backend_required_invalid)"
        ))),
    }
}

pub(super) fn resolve_kolme_live_managed_signer_timeout_seconds() -> Result<u64, ConfigError> {
    match env::var(KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_ENV) {
        Ok(raw_timeout) => {
            let trimmed = raw_timeout.trim();
            if trimmed.is_empty() {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "{KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_ENV} must not be empty when present (managed_signer_backend_timeout_invalid)"
                )));
            }
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
        Err(env::VarError::NotPresent) => Ok(KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_DEFAULT),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_ENV} must be valid utf-8 when present (managed_signer_backend_timeout_invalid)"
        ))),
    }
}
