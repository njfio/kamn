use super::{ConfigError, LocalProfile, NodeRole, SyncMode};

const OBSERVABILITY_METRICS_PATH_ERROR: &str =
    "observability endpoint metrics path must start with '/'";
const OBSERVABILITY_HEALTH_PATH_ERROR: &str =
    "observability endpoint health path must start with '/'";

pub(super) struct ProfileDefaultsInputs<'a> {
    pub(super) profile: Option<LocalProfile>,
    pub(super) role: &'a mut Option<NodeRole>,
    pub(super) chain_id: &'a mut String,
    pub(super) chain_version: &'a mut String,
    pub(super) storage_dir: &'a mut String,
    pub(super) enable_gossip: &'a mut bool,
    pub(super) sync_mode: &'a mut SyncMode,
    pub(super) role_overridden: bool,
    pub(super) chain_id_overridden: bool,
    pub(super) chain_version_overridden: bool,
    pub(super) storage_dir_overridden: bool,
    pub(super) gossip_overridden: bool,
    pub(super) sync_mode_overridden: bool,
}

pub(super) fn apply_profile_defaults(inputs: ProfileDefaultsInputs<'_>) {
    let Some(selected_profile) = inputs.profile else {
        return;
    };
    if !inputs.role_overridden {
        *inputs.role = Some(selected_profile.default_role());
    }
    if !inputs.chain_id_overridden {
        *inputs.chain_id = "kamn-localnet".to_owned();
    }
    if !inputs.chain_version_overridden {
        *inputs.chain_version = "v0.1.0".to_owned();
    }
    if !inputs.storage_dir_overridden {
        *inputs.storage_dir = selected_profile.default_storage_dir().to_owned();
    }
    if !inputs.gossip_overridden {
        *inputs.enable_gossip = true;
    }
    if !inputs.sync_mode_overridden {
        *inputs.sync_mode = SyncMode::Fast;
    }
}

pub(super) struct EndpointGuardInputs<'a> {
    pub(super) api_bind_addr_present: bool,
    pub(super) api_max_requests_overridden: bool,
    pub(super) api_idle_timeout_ms_overridden: bool,
    pub(super) api_body_limit_bytes_overridden: bool,
    pub(super) api_concurrency_limit_overridden: bool,
    pub(super) api_rate_limit_per_second_overridden: bool,
    pub(super) observability_endpoint_bind_addr_present: bool,
    pub(super) observability_endpoint_metrics_path_overridden: bool,
    pub(super) observability_endpoint_health_path_overridden: bool,
    pub(super) observability_endpoint_max_requests_overridden: bool,
    pub(super) observability_endpoint_idle_timeout_ms_overridden: bool,
    pub(super) observability_endpoint_metrics_path: &'a str,
    pub(super) observability_endpoint_health_path: &'a str,
}

pub(super) fn validate_endpoint_guards(inputs: EndpointGuardInputs<'_>) -> Result<(), ConfigError> {
    validate_api_bind_override_guard(&inputs)?;
    validate_observability_bind_override_guard(&inputs)?;
    validate_observability_path_guards(&inputs)?;
    Ok(())
}

fn validate_api_bind_override_guard(inputs: &EndpointGuardInputs<'_>) -> Result<(), ConfigError> {
    if !inputs.api_bind_addr_present
        && (inputs.api_max_requests_overridden
            || inputs.api_idle_timeout_ms_overridden
            || inputs.api_body_limit_bytes_overridden
            || inputs.api_concurrency_limit_overridden
            || inputs.api_rate_limit_per_second_overridden)
    {
        return Err(ConfigError::MissingArgumentValue("--api-bind"));
    }
    Ok(())
}

fn validate_observability_bind_override_guard(
    inputs: &EndpointGuardInputs<'_>,
) -> Result<(), ConfigError> {
    if !inputs.observability_endpoint_bind_addr_present
        && (inputs.observability_endpoint_metrics_path_overridden
            || inputs.observability_endpoint_health_path_overridden
            || inputs.observability_endpoint_max_requests_overridden
            || inputs.observability_endpoint_idle_timeout_ms_overridden)
    {
        return Err(ConfigError::MissingArgumentValue(
            "--observability-endpoint-bind",
        ));
    }
    Ok(())
}

fn validate_observability_path_guards(inputs: &EndpointGuardInputs<'_>) -> Result<(), ConfigError> {
    if !inputs.observability_endpoint_bind_addr_present {
        return Ok(());
    }
    validate_path_has_leading_slash(
        inputs.observability_endpoint_metrics_path,
        OBSERVABILITY_METRICS_PATH_ERROR,
    )?;
    validate_path_has_leading_slash(
        inputs.observability_endpoint_health_path,
        OBSERVABILITY_HEALTH_PATH_ERROR,
    )?;
    Ok(())
}

fn validate_path_has_leading_slash(path: &str, error_message: &str) -> Result<(), ConfigError> {
    if !path.starts_with('/') {
        return Err(ConfigError::RuntimeDaemonLifecycle(
            error_message.to_owned(),
        ));
    }
    Ok(())
}
