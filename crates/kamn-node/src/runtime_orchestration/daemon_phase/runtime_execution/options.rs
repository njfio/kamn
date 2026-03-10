use super::super::super::*;

pub(super) struct ParsedDaemonRuntimeOptions {
    pub(super) max_ticks: u64,
    pub(super) tick_interval_ms: u64,
    pub(super) daemon_shutdown_signal_ticks: Vec<u64>,
    pub(super) daemon_shutdown_os_signals: bool,
    pub(super) daemon_shutdown_drain_ticks: Option<u64>,
    pub(super) daemon_shutdown_timeout_ticks: Option<u64>,
    pub(super) daemon_peer_id: Option<String>,
    pub(super) daemon_lifecycle_events: Vec<PeerLifecycleEvent>,
    pub(super) service_api_state_file: Option<String>,
    pub(super) service_api_relay_spool_file: Option<String>,
    pub(super) service_api_signature_state_hash: String,
}

pub(super) fn parse_daemon_runtime_options(
    options: DaemonRuntimeOptions,
) -> Result<ParsedDaemonRuntimeOptions, ConfigError> {
    let DaemonRuntimeOptions {
        daemon_max_ticks,
        daemon_tick_interval_ms,
        daemon_shutdown_signal_ticks,
        daemon_shutdown_os_signals,
        daemon_shutdown_drain_ticks,
        daemon_shutdown_timeout_ticks,
        daemon_peer_id,
        daemon_lifecycle_events,
        service_api_state_file,
        service_api_relay_spool_file,
        service_api_signature_state_hash,
    } = options;
    Ok(ParsedDaemonRuntimeOptions {
        max_ticks: required_u64_option(daemon_max_ticks, "--daemon-max-ticks")?,
        tick_interval_ms: required_u64_option(
            daemon_tick_interval_ms,
            "--daemon-tick-interval-ms",
        )?,
        daemon_shutdown_signal_ticks,
        daemon_shutdown_os_signals,
        daemon_shutdown_drain_ticks,
        daemon_shutdown_timeout_ticks,
        daemon_peer_id,
        daemon_lifecycle_events,
        service_api_state_file,
        service_api_relay_spool_file,
        service_api_signature_state_hash,
    })
}

fn required_u64_option(value: Option<u64>, flag: &'static str) -> Result<u64, ConfigError> {
    value.ok_or(ConfigError::MissingArgumentValue(flag))
}
