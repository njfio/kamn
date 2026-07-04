use super::super::super::super::*;
use super::super::spool_io::{drain_relay_entries, project_relayed_state, requeue_failed_entries};

#[derive(Default)]
pub(super) struct RelayBatchOutcome {
    pub(super) relay_message_ids: Vec<String>,
    pub(super) failed_entries: Vec<super::RelayEntry>,
}

pub(super) fn build_relay_batch(
    runtime_processing: &mut crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    service_api_relay_spool_file: Option<&str>,
    relay_p2p_context: Option<&super::RelayP2pContext>,
    relay_route_map: &super::RelayRouteMap,
    relay_signing_private_key_hex: Option<&str>,
    relay_nonce_counter: &mut u64,
    service_api_signature_state_hash: &str,
) -> Result<RelayBatchOutcome, ConfigError> {
    let relay_entries = count_drained_entries(runtime_processing, service_api_relay_spool_file)?;
    super::process_relay_entries(
        runtime_processing,
        relay_entries,
        relay_p2p_context,
        relay_route_map,
        relay_signing_private_key_hex,
        relay_nonce_counter,
        service_api_signature_state_hash,
    )
}

pub(super) fn finalize_relay_batch(
    runtime_processing: &mut crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    service_api_state_file: Option<&str>,
    service_api_relay_spool_file: Option<&str>,
    batch: RelayBatchOutcome,
) -> Result<(), ConfigError> {
    requeue_failed_entries(service_api_relay_spool_file, batch.failed_entries)?;
    project_relayed_state(
        runtime_processing,
        service_api_state_file,
        batch.relay_message_ids.as_slice(),
    )
}

fn count_drained_entries(
    runtime_processing: &mut crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    service_api_relay_spool_file: Option<&str>,
) -> Result<Vec<super::RelayEntry>, ConfigError> {
    let relay_entries = drain_relay_entries(service_api_relay_spool_file)?;
    runtime_processing.relay_drained_count = runtime_processing
        .relay_drained_count
        .saturating_add(relay_entries.len() as u64);
    Ok(relay_entries)
}
