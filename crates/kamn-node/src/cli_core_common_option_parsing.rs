use super::{
    ConfigError, DiagnosticsMode, LocalProfile, NodeRole, OutputMode, RuntimeMode, SyncMode,
};

pub(super) struct CoreCommonOptionState<'a> {
    pub(super) role: &'a mut Option<NodeRole>,
    pub(super) profile: &'a mut Option<LocalProfile>,
    pub(super) chain_id: &'a mut String,
    pub(super) chain_version: &'a mut String,
    pub(super) storage_dir: &'a mut String,
    pub(super) enable_gossip: &'a mut bool,
    pub(super) sync_mode: &'a mut SyncMode,
    pub(super) runtime_mode: &'a mut RuntimeMode,
    pub(super) output_mode: &'a mut OutputMode,
    pub(super) diagnostics_mode: &'a mut DiagnosticsMode,
    pub(super) role_overridden: &'a mut bool,
    pub(super) chain_id_overridden: &'a mut bool,
    pub(super) chain_version_overridden: &'a mut bool,
    pub(super) storage_dir_overridden: &'a mut bool,
    pub(super) gossip_overridden: &'a mut bool,
    pub(super) sync_mode_overridden: &'a mut bool,
}

pub(super) fn try_parse_core_common_option(
    arg: &str,
    iter: &mut std::vec::IntoIter<String>,
    state: &mut CoreCommonOptionState<'_>,
) -> Result<bool, ConfigError> {
    if try_parse_role_profile_option(arg, iter, state)? {
        return Ok(true);
    }
    if try_parse_chain_storage_option(arg, iter, state)? {
        return Ok(true);
    }
    if try_parse_mode_output_option(arg, iter, state)? {
        return Ok(true);
    }
    Ok(false)
}

fn try_parse_role_profile_option(
    arg: &str,
    iter: &mut std::vec::IntoIter<String>,
    state: &mut CoreCommonOptionState<'_>,
) -> Result<bool, ConfigError> {
    match arg {
        "--role" => {
            let value = read_required_arg(iter, "--role")?;
            *state.role = Some(value.parse::<NodeRole>()?);
            *state.role_overridden = true;
            Ok(true)
        }
        "--profile" => {
            let value = read_required_arg(iter, "--profile")?;
            *state.profile = Some(LocalProfile::parse(&value)?);
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn try_parse_chain_storage_option(
    arg: &str,
    iter: &mut std::vec::IntoIter<String>,
    state: &mut CoreCommonOptionState<'_>,
) -> Result<bool, ConfigError> {
    if try_parse_chain_identity_option(arg, iter, state)? {
        return Ok(true);
    }
    try_parse_storage_sync_option(arg, iter, state)
}

fn try_parse_chain_identity_option(
    arg: &str,
    iter: &mut std::vec::IntoIter<String>,
    state: &mut CoreCommonOptionState<'_>,
) -> Result<bool, ConfigError> {
    match arg {
        "--chain-id" => {
            *state.chain_id = read_required_arg(iter, "--chain-id")?;
            *state.chain_id_overridden = true;
            Ok(true)
        }
        "--chain-version" => {
            *state.chain_version = read_required_arg(iter, "--chain-version")?;
            *state.chain_version_overridden = true;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn try_parse_storage_sync_option(
    arg: &str,
    iter: &mut std::vec::IntoIter<String>,
    state: &mut CoreCommonOptionState<'_>,
) -> Result<bool, ConfigError> {
    match arg {
        "--storage-dir" => {
            *state.storage_dir = read_required_arg(iter, "--storage-dir")?;
            *state.storage_dir_overridden = true;
            Ok(true)
        }
        "--disable-gossip" => {
            *state.enable_gossip = false;
            *state.gossip_overridden = true;
            Ok(true)
        }
        "--sync-mode" => {
            let value = read_required_arg(iter, "--sync-mode")?;
            *state.sync_mode = value.parse::<SyncMode>()?;
            *state.sync_mode_overridden = true;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn try_parse_mode_output_option(
    arg: &str,
    iter: &mut std::vec::IntoIter<String>,
    state: &mut CoreCommonOptionState<'_>,
) -> Result<bool, ConfigError> {
    match arg {
        "--runtime-mode" => {
            let value = read_required_arg(iter, "--runtime-mode")?;
            *state.runtime_mode = RuntimeMode::parse(&value)?;
            Ok(true)
        }
        "--output" => {
            let value = read_required_arg(iter, "--output")?;
            *state.output_mode = OutputMode::parse(&value)?;
            Ok(true)
        }
        "--diagnostics" => {
            let value = read_required_arg(iter, "--diagnostics")?;
            *state.diagnostics_mode = DiagnosticsMode::parse(&value)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn read_required_arg(
    iter: &mut std::vec::IntoIter<String>,
    flag: &'static str,
) -> Result<String, ConfigError> {
    iter.next().ok_or(ConfigError::MissingArgumentValue(flag))
}
