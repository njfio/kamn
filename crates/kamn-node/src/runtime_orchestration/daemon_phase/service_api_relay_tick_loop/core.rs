mod tick_iteration;

use super::super::super::*;
use super::super::service_api_relay_p2p::{
    drain_daemon_service_api_relay_p2p_inbox, resolve_daemon_service_api_relay_p2p_context,
};
use super::env_support::{
    initial_daemon_relay_nonce_counter, resolve_daemon_service_api_auth_private_key_hex,
    resolve_daemon_service_api_relay_recipient_route_map,
};
use super::forwarding::{process_relay_spool, RelaySpoolArgs};
use std::time::Duration;
use tick_iteration::execute_tick;

pub(super) fn execute_daemon_service_api_relay_tick_loop(
    executed_ticks: u64,
    tick_interval_ms: u64,
    service_api_state_file: Option<&str>,
    service_api_relay_spool_file: Option<&str>,
    service_api_signature_state_hash: &str,
) -> Result<crate::daemon_observability::DaemonRuntimeProcessingTelemetry, ConfigError> {
    let mut runtime_processing = default_runtime_processing(executed_ticks);
    let mut tick_context = build_tick_context(
        executed_ticks,
        tick_interval_ms,
        service_api_relay_spool_file,
    )?;
    if tick_context.executed_ticks_is_zero() {
        return Ok(runtime_processing);
    }
    run_tick_loop(
        &mut runtime_processing,
        &mut tick_context,
        service_api_state_file,
        service_api_relay_spool_file,
        service_api_signature_state_hash,
    )?;
    Ok(runtime_processing)
}

struct TickContext {
    relay_enabled: bool,
    relay_p2p_context: Option<super::super::service_api_relay_p2p::DaemonServiceApiRelayP2pContext>,
    relay_route_map: std::collections::BTreeMap<String, String>,
    relay_signing_private_key_hex: Option<String>,
    relay_nonce_counter: u64,
    tick_duration: Duration,
    executed_ticks: u64,
}

impl TickContext {
    fn executed_ticks_is_zero(&self) -> bool {
        self.executed_ticks == 0
    }
}

fn default_runtime_processing(
    executed_ticks: u64,
) -> crate::daemon_observability::DaemonRuntimeProcessingTelemetry {
    crate::daemon_observability::DaemonRuntimeProcessingTelemetry {
        executed_ticks,
        ..crate::daemon_observability::DaemonRuntimeProcessingTelemetry::default()
    }
}

fn build_tick_context(
    executed_ticks: u64,
    tick_interval_ms: u64,
    service_api_relay_spool_file: Option<&str>,
) -> Result<TickContext, ConfigError> {
    let relay_route_map = resolve_daemon_service_api_relay_recipient_route_map()?;
    Ok(TickContext {
        relay_enabled: service_api_relay_spool_file.is_some(),
        relay_p2p_context: resolve_daemon_service_api_relay_p2p_context()?,
        relay_signing_private_key_hex: resolve_optional_relay_signing_key(&relay_route_map)?,
        relay_nonce_counter: initial_daemon_relay_nonce_counter(),
        relay_route_map,
        tick_duration: Duration::from_millis(tick_interval_ms.max(1)),
        executed_ticks,
    })
}

fn run_tick_loop(
    runtime_processing: &mut crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    tick_context: &mut TickContext,
    service_api_state_file: Option<&str>,
    service_api_relay_spool_file: Option<&str>,
    service_api_signature_state_hash: &str,
) -> Result<(), ConfigError> {
    for tick in 0..tick_context.executed_ticks {
        execute_tick(
            runtime_processing,
            tick_context,
            tick,
            service_api_state_file,
            service_api_relay_spool_file,
            service_api_signature_state_hash,
        )?;
    }
    Ok(())
}

pub(super) fn daemon_tick_remaining_sleep_duration(
    tick: u64,
    executed_ticks: u64,
    tick_duration: Duration,
    elapsed: Duration,
) -> Option<Duration> {
    if tick + 1 >= executed_ticks {
        return None;
    }
    let remaining = tick_duration.checked_sub(elapsed)?;
    if remaining.is_zero() {
        return None;
    }
    Some(remaining)
}

fn resolve_optional_relay_signing_key(
    relay_route_map: &std::collections::BTreeMap<String, String>,
) -> Result<Option<String>, ConfigError> {
    if relay_route_map.is_empty() {
        return Ok(None);
    }
    resolve_daemon_service_api_auth_private_key_hex().map(Some)
}

pub(super) fn ingest_p2p_inbox(
    relay_p2p_context: &super::super::service_api_relay_p2p::DaemonServiceApiRelayP2pContext,
    service_api_state_file: Option<&str>,
) -> Result<(), ConfigError> {
    let p2p_ingested_count =
        drain_daemon_service_api_relay_p2p_inbox(relay_p2p_context, service_api_state_file)
            .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;
    let ingested_count_label = p2p_ingested_count.to_string();
    log_info(
        "node.runtime.daemon.relay.p2p.ingested",
        &[("ingested_count", ingested_count_label.as_str())],
    )
    .map_err(|logging_error| ConfigError::RuntimeDaemonLifecycle(logging_error.to_string()))
}

pub(super) fn record_tick_sample(
    runtime_processing: &mut crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    elapsed: Duration,
) {
    let elapsed_ms = elapsed.as_millis();
    runtime_processing
        .tick_processing_samples_ms
        .push((elapsed_ms.min(u128::from(u64::MAX)) as u64).max(1));
}

pub(super) fn sleep_remaining_tick_budget(
    runtime_processing: &mut crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    tick: u64,
    executed_ticks: u64,
    tick_duration: Duration,
    elapsed: Duration,
) {
    if let Some(remaining_sleep) =
        daemon_tick_remaining_sleep_duration(tick, executed_ticks, tick_duration, elapsed)
    {
        std::thread::sleep(remaining_sleep);
        runtime_processing.tick_sleep_count = runtime_processing.tick_sleep_count.saturating_add(1);
    }
}
