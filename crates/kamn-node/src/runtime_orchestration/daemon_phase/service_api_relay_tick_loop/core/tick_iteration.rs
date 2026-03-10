use super::super::super::super::*;
use std::time::Instant;

pub(super) fn execute_tick(
    runtime_processing: &mut crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    tick_context: &mut super::TickContext,
    tick: u64,
    service_api_state_file: Option<&str>,
    service_api_relay_spool_file: Option<&str>,
    service_api_signature_state_hash: &str,
) -> Result<(), ConfigError> {
    let tick_started_at = Instant::now();
    process_tick_io(
        runtime_processing,
        tick_context,
        service_api_state_file,
        service_api_relay_spool_file,
        service_api_signature_state_hash,
    )?;
    finalize_tick(
        runtime_processing,
        tick,
        tick_context,
        tick_started_at.elapsed(),
    );
    Ok(())
}

fn process_tick_io(
    runtime_processing: &mut crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    tick_context: &mut super::TickContext,
    service_api_state_file: Option<&str>,
    service_api_relay_spool_file: Option<&str>,
    service_api_signature_state_hash: &str,
) -> Result<(), ConfigError> {
    ingest_p2p_inbox_if_present(
        tick_context.relay_p2p_context.as_ref(),
        service_api_state_file,
    )?;
    forward_relay_spool_if_enabled(
        runtime_processing,
        tick_context,
        service_api_state_file,
        service_api_relay_spool_file,
        service_api_signature_state_hash,
    )
}

fn ingest_p2p_inbox_if_present(
    relay_p2p_context: Option<
        &super::super::super::service_api_relay_p2p::DaemonServiceApiRelayP2pContext,
    >,
    service_api_state_file: Option<&str>,
) -> Result<(), ConfigError> {
    if let Some(relay_p2p_context) = relay_p2p_context {
        super::ingest_p2p_inbox(relay_p2p_context, service_api_state_file)?;
    }
    Ok(())
}

fn forward_relay_spool_if_enabled(
    runtime_processing: &mut crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    tick_context: &mut super::TickContext,
    service_api_state_file: Option<&str>,
    service_api_relay_spool_file: Option<&str>,
    service_api_signature_state_hash: &str,
) -> Result<(), ConfigError> {
    if !tick_context.relay_enabled {
        return Ok(());
    }
    super::process_relay_spool(
        runtime_processing,
        tick_context.relay_p2p_context.as_ref(),
        &tick_context.relay_route_map,
        tick_context.relay_signing_private_key_hex.as_deref(),
        &mut tick_context.relay_nonce_counter,
        service_api_state_file,
        service_api_relay_spool_file,
        service_api_signature_state_hash,
    )
}

fn finalize_tick(
    runtime_processing: &mut crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    tick: u64,
    tick_context: &super::TickContext,
    elapsed: std::time::Duration,
) {
    super::record_tick_sample(runtime_processing, elapsed);
    super::sleep_remaining_tick_budget(
        runtime_processing,
        tick,
        tick_context.executed_ticks,
        tick_context.tick_duration,
        elapsed,
    );
}
