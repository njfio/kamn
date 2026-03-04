use super::{ConfigError, RuntimeMode, RuntimeModeKind};

#[path = "cli_runtime_mode_validation/kolme_live.rs"]
mod kolme_live;

use kolme_live::validate_kolme_live_mode;

pub(super) struct RuntimeModeValidationInputs<'a> {
    pub(super) runtime_mode: RuntimeMode,
    pub(super) expected_state_version: Option<u64>,
    pub(super) expected_state_hash: Option<&'a str>,
    pub(super) proposals_len: usize,
    pub(super) rejoin_attempts_len: usize,
    pub(super) daemon_max_ticks: Option<u64>,
    pub(super) daemon_tick_interval_ms: Option<u64>,
    pub(super) daemon_shutdown_signal_ticks_len: usize,
    pub(super) daemon_shutdown_os_signals: bool,
    pub(super) daemon_shutdown_drain_ticks: Option<u64>,
    pub(super) daemon_shutdown_timeout_ticks: Option<u64>,
    pub(super) daemon_peer_id_present: bool,
    pub(super) daemon_lifecycle_events_len: usize,
    pub(super) api_bind_addr_present: bool,
    pub(super) kolme_live_base_url: Option<&'a str>,
    pub(super) kolme_live_provider_hint: Option<&'a str>,
    pub(super) kolme_live_signing_profile: Option<&'a str>,
    pub(super) kolme_live_strict_signer_contracts: bool,
    pub(super) kolme_live_signer_profile: Option<&'a str>,
    pub(super) kolme_live_signer_key_source: Option<&'a str>,
}

pub(super) fn validate_runtime_mode_requirements(
    inputs: RuntimeModeValidationInputs<'_>,
) -> Result<(), ConfigError> {
    match inputs.runtime_mode.kind {
        RuntimeModeKind::Planning => {
            validate_planning_mode(inputs.expected_state_hash, inputs.proposals_len)
        }
        RuntimeModeKind::RecoveryCheck => validate_recovery_mode(
            inputs.expected_state_version,
            inputs.expected_state_hash,
            inputs.rejoin_attempts_len,
        ),
        RuntimeModeKind::Daemon | RuntimeModeKind::Full => validate_daemon_or_full_mode(&inputs),
        RuntimeModeKind::Api => validate_api_mode(inputs.api_bind_addr_present),
        RuntimeModeKind::KolmeLive => validate_kolme_live_mode(&inputs),
        RuntimeModeKind::Bootstrap => Ok(()),
    }
}

fn validate_planning_mode(
    expected_state_hash: Option<&str>,
    proposals_len: usize,
) -> Result<(), ConfigError> {
    if expected_state_hash.is_none() {
        return Err(ConfigError::MissingArgumentValue("--expected-state-hash"));
    }
    if proposals_len == 0 {
        return Err(ConfigError::MissingArgumentValue("--proposal"));
    }
    Ok(())
}

fn validate_recovery_mode(
    expected_state_version: Option<u64>,
    expected_state_hash: Option<&str>,
    rejoin_attempts_len: usize,
) -> Result<(), ConfigError> {
    if expected_state_version.is_none() {
        return Err(ConfigError::MissingArgumentValue(
            "--expected-state-version",
        ));
    }
    if expected_state_hash.is_none() {
        return Err(ConfigError::MissingArgumentValue("--expected-state-hash"));
    }
    if rejoin_attempts_len == 0 {
        return Err(ConfigError::MissingArgumentValue("--rejoin-attempt"));
    }
    Ok(())
}

fn validate_daemon_or_full_mode(
    inputs: &RuntimeModeValidationInputs<'_>,
) -> Result<(), ConfigError> {
    validate_daemon_tick_requirements(inputs.daemon_max_ticks, inputs.daemon_tick_interval_ms)?;
    validate_daemon_lifecycle_requirements(
        inputs.daemon_lifecycle_events_len,
        inputs.daemon_peer_id_present,
    )?;
    validate_daemon_shutdown_requirements(
        inputs.daemon_shutdown_signal_ticks_len,
        inputs.daemon_shutdown_os_signals,
        inputs.daemon_shutdown_drain_ticks,
        inputs.daemon_shutdown_timeout_ticks,
    )?;
    validate_api_mode(inputs.api_bind_addr_present)?;
    Ok(())
}

fn validate_daemon_tick_requirements(
    daemon_max_ticks: Option<u64>,
    daemon_tick_interval_ms: Option<u64>,
) -> Result<(), ConfigError> {
    if daemon_max_ticks.is_none() {
        return Err(ConfigError::MissingArgumentValue("--daemon-max-ticks"));
    }
    if daemon_tick_interval_ms.is_none() {
        return Err(ConfigError::MissingArgumentValue(
            "--daemon-tick-interval-ms",
        ));
    }
    Ok(())
}

fn validate_daemon_lifecycle_requirements(
    daemon_lifecycle_events_len: usize,
    daemon_peer_id_present: bool,
) -> Result<(), ConfigError> {
    if daemon_lifecycle_events_len > 0 && !daemon_peer_id_present {
        return Err(ConfigError::MissingArgumentValue("--daemon-peer-id"));
    }
    Ok(())
}

fn validate_daemon_shutdown_requirements(
    daemon_shutdown_signal_ticks_len: usize,
    daemon_shutdown_os_signals: bool,
    daemon_shutdown_drain_ticks: Option<u64>,
    daemon_shutdown_timeout_ticks: Option<u64>,
) -> Result<(), ConfigError> {
    if daemon_shutdown_signal_ticks_len > 0 {
        if daemon_shutdown_drain_ticks.is_none() {
            return Err(ConfigError::MissingArgumentValue(
                "--daemon-shutdown-drain-ticks",
            ));
        }
        if daemon_shutdown_timeout_ticks.is_none() {
            return Err(ConfigError::MissingArgumentValue(
                "--daemon-shutdown-timeout-ticks",
            ));
        }
        return Ok(());
    }
    if (daemon_shutdown_drain_ticks.is_some() || daemon_shutdown_timeout_ticks.is_some())
        && !daemon_shutdown_os_signals
    {
        return Err(ConfigError::MissingArgumentValue(
            "--daemon-shutdown-signal-tick",
        ));
    }
    Ok(())
}

fn validate_api_mode(api_bind_addr_present: bool) -> Result<(), ConfigError> {
    if !api_bind_addr_present {
        return Err(ConfigError::MissingArgumentValue("--api-bind"));
    }
    Ok(())
}
