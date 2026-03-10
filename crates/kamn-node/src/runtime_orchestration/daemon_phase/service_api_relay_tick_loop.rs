mod core;
mod env_support;
mod forwarding;
mod http_forward;
mod spool_io;

use super::super::*;

pub(super) fn execute_daemon_service_api_relay_tick_loop(
    executed_ticks: u64,
    tick_interval_ms: u64,
    service_api_state_file: Option<&str>,
    service_api_relay_spool_file: Option<&str>,
    service_api_signature_state_hash: &str,
) -> Result<crate::daemon_observability::DaemonRuntimeProcessingTelemetry, ConfigError> {
    core::execute_daemon_service_api_relay_tick_loop(
        executed_ticks,
        tick_interval_ms,
        service_api_state_file,
        service_api_relay_spool_file,
        service_api_signature_state_hash,
    )
}

#[cfg(test)]
pub(super) fn daemon_tick_remaining_sleep_duration(
    tick: u64,
    executed_ticks: u64,
    tick_duration: std::time::Duration,
    elapsed: std::time::Duration,
) -> Option<std::time::Duration> {
    core::daemon_tick_remaining_sleep_duration(tick, executed_ticks, tick_duration, elapsed)
}

#[cfg(test)]
pub(super) const SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV_FOR_TEST: &str =
    env_support::SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV_FOR_TEST;
