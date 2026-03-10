mod decisions;

use super::super::super::*;
use super::super::service_api_relay_p2p::forward_service_api_relay_entry_via_p2p;
use super::http_forward::forward_service_api_relay_entry;
use super::spool_io::{drain_relay_entries, project_relayed_state, requeue_failed_entries};
use decisions::{
    combine_forward_errors, forward_single_entry, missing_signing_key_error, p2p_forward_succeeded,
    retain_pending_or_failed, try_p2p_forward,
};

pub(super) fn process_relay_spool(
    runtime_processing: &mut crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    relay_p2p_context: Option<
        &super::super::service_api_relay_p2p::DaemonServiceApiRelayP2pContext,
    >,
    relay_route_map: &std::collections::BTreeMap<String, String>,
    relay_signing_private_key_hex: Option<&str>,
    relay_nonce_counter: &mut u64,
    service_api_state_file: Option<&str>,
    service_api_relay_spool_file: Option<&str>,
    service_api_signature_state_hash: &str,
) -> Result<(), ConfigError> {
    let relay_entries = count_drained_entries(runtime_processing, service_api_relay_spool_file)?;
    let batch = handle_relay_entries(
        runtime_processing,
        relay_entries,
        relay_p2p_context,
        relay_route_map,
        relay_signing_private_key_hex,
        relay_nonce_counter,
        service_api_signature_state_hash,
    )?;
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
) -> Result<Vec<crate::service_api_endpoint::ServiceApiRelaySpoolEntry>, ConfigError> {
    let relay_entries = drain_relay_entries(service_api_relay_spool_file)?;
    runtime_processing.relay_drained_count = runtime_processing
        .relay_drained_count
        .saturating_add(relay_entries.len() as u64);
    Ok(relay_entries)
}

fn handle_relay_entries(
    runtime_processing: &mut crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    relay_entries: Vec<crate::service_api_endpoint::ServiceApiRelaySpoolEntry>,
    relay_p2p_context: Option<
        &super::super::service_api_relay_p2p::DaemonServiceApiRelayP2pContext,
    >,
    relay_route_map: &std::collections::BTreeMap<String, String>,
    relay_signing_private_key_hex: Option<&str>,
    relay_nonce_counter: &mut u64,
    service_api_signature_state_hash: &str,
) -> Result<RelayBatchOutcome, ConfigError> {
    let mut relay_message_ids = Vec::new();
    let mut failed_entries = Vec::new();
    for relay_entry in relay_entries {
        apply_relay_decision(
            runtime_processing,
            &mut relay_message_ids,
            &mut failed_entries,
            relay_p2p_context,
            relay_route_map,
            relay_signing_private_key_hex,
            relay_nonce_counter,
            service_api_signature_state_hash,
            relay_entry,
        )?;
    }
    Ok(RelayBatchOutcome {
        relay_message_ids,
        failed_entries,
    })
}

enum RelayForwardDecision {
    Forwarded,
    RetainPending,
    Failed(String),
}

struct RelayBatchOutcome {
    relay_message_ids: Vec<String>,
    failed_entries: Vec<crate::service_api_endpoint::ServiceApiRelaySpoolEntry>,
}

fn apply_relay_decision(
    runtime_processing: &mut crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    relay_message_ids: &mut Vec<String>,
    failed_entries: &mut Vec<crate::service_api_endpoint::ServiceApiRelaySpoolEntry>,
    relay_p2p_context: Option<
        &super::super::service_api_relay_p2p::DaemonServiceApiRelayP2pContext,
    >,
    relay_route_map: &std::collections::BTreeMap<String, String>,
    relay_signing_private_key_hex: Option<&str>,
    relay_nonce_counter: &mut u64,
    service_api_signature_state_hash: &str,
    relay_entry: crate::service_api_endpoint::ServiceApiRelaySpoolEntry,
) -> Result<(), ConfigError> {
    match forward_single_entry(
        relay_p2p_context,
        relay_route_map,
        relay_signing_private_key_hex,
        relay_nonce_counter,
        service_api_signature_state_hash,
        &relay_entry,
    )? {
        RelayForwardDecision::Forwarded => relay_message_ids.push(relay_entry.message_id.clone()),
        RelayForwardDecision::RetainPending => failed_entries.push(relay_entry),
        RelayForwardDecision::Failed(error_message) => {
            record_forward_failure(runtime_processing, &relay_entry, &error_message)?;
            failed_entries.push(relay_entry);
        }
    }
    Ok(())
}

fn forward_via_http(
    relay_route_map: &std::collections::BTreeMap<String, String>,
    relay_entry: &crate::service_api_endpoint::ServiceApiRelaySpoolEntry,
    service_api_signature_state_hash: &str,
    signing_key_hex: &str,
    relay_nonce_counter: &mut u64,
    p2p_error: Option<String>,
) -> Result<RelayForwardDecision, ConfigError> {
    match forward_service_api_relay_entry(
        relay_route_map,
        relay_entry,
        service_api_signature_state_hash,
        signing_key_hex,
        relay_nonce_counter,
    ) {
        Ok(()) => Ok(RelayForwardDecision::Forwarded),
        Err(error) => Ok(RelayForwardDecision::Failed(combine_forward_errors(
            p2p_error, error,
        ))),
    }
}

fn record_forward_failure(
    runtime_processing: &mut crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    relay_entry: &crate::service_api_endpoint::ServiceApiRelaySpoolEntry,
    error_message: &str,
) -> Result<(), ConfigError> {
    runtime_processing.processing_error_count =
        runtime_processing.processing_error_count.saturating_add(1);
    let queued_at_label = relay_entry.queued_at_unix.to_string();
    log_info(
        "node.runtime.daemon.relay.forward.failed",
        &[
            ("message_id", relay_entry.message_id.as_str()),
            ("recipient_did", relay_entry.recipient_did.as_str()),
            ("queued_at_unix", queued_at_label.as_str()),
            ("error", error_message),
        ],
    )
    .map_err(|logging_error| ConfigError::RuntimeDaemonLifecycle(logging_error.to_string()))
}
