mod batch;
mod decisions;

use super::super::super::*;
use super::http_forward::forward_service_api_relay_entry;
use batch::{build_relay_batch, finalize_relay_batch, RelayBatchOutcome};
use decisions::{combine_forward_errors, forward_single_entry};

type RelayEntry = crate::service_api_endpoint::ServiceApiRelaySpoolEntry;
type RelayRouteMap = std::collections::BTreeMap<String, String>;
type RelayP2pContext = super::super::service_api_relay_p2p::DaemonServiceApiRelayP2pContext;
type RuntimeProcessing = crate::daemon_observability::DaemonRuntimeProcessingTelemetry;

struct RelaySpoolArgs<'a> {
    relay_p2p_context: Option<&'a RelayP2pContext>,
    relay_route_map: &'a RelayRouteMap,
    relay_signing_private_key_hex: Option<&'a str>,
    relay_nonce_counter: &'a mut u64,
    service_api_state_file: Option<&'a str>,
    service_api_relay_spool_file: Option<&'a str>,
    service_api_signature_state_hash: &'a str,
}

pub(super) fn process_relay_spool(
    runtime_processing: &mut RuntimeProcessing,
    relay_p2p_context: Option<&RelayP2pContext>,
    relay_route_map: &RelayRouteMap,
    relay_signing_private_key_hex: Option<&str>,
    relay_nonce_counter: &mut u64,
    service_api_state_file: Option<&str>,
    service_api_relay_spool_file: Option<&str>,
    service_api_signature_state_hash: &str,
) -> Result<(), ConfigError> {
    process_relay_spool_with_args(
        runtime_processing,
        RelaySpoolArgs {
            relay_p2p_context,
            relay_route_map,
            relay_signing_private_key_hex,
            relay_nonce_counter,
            service_api_state_file,
            service_api_relay_spool_file,
            service_api_signature_state_hash,
        },
    )
}

fn process_relay_spool_with_args(
    runtime_processing: &mut RuntimeProcessing,
    args: RelaySpoolArgs<'_>,
) -> Result<(), ConfigError> {
    let batch = build_relay_batch(
        runtime_processing,
        args.service_api_relay_spool_file,
        args.relay_p2p_context,
        args.relay_route_map,
        args.relay_signing_private_key_hex,
        args.relay_nonce_counter,
        args.service_api_signature_state_hash,
    )?;
    finalize_relay_batch(
        runtime_processing,
        args.service_api_state_file,
        args.service_api_relay_spool_file,
        batch,
    )
}

fn process_relay_entries(
    runtime_processing: &mut crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    relay_entries: Vec<RelayEntry>,
    relay_p2p_context: Option<&RelayP2pContext>,
    relay_route_map: &RelayRouteMap,
    relay_signing_private_key_hex: Option<&str>,
    relay_nonce_counter: &mut u64,
    service_api_signature_state_hash: &str,
) -> Result<RelayBatchOutcome, ConfigError> {
    let mut batch = RelayBatchOutcome::default();
    for relay_entry in relay_entries {
        apply_relay_decision(
            runtime_processing,
            &mut batch,
            relay_p2p_context,
            relay_route_map,
            relay_signing_private_key_hex,
            relay_nonce_counter,
            service_api_signature_state_hash,
            relay_entry,
        )?;
    }
    Ok(batch)
}

enum RelayForwardDecision {
    Forwarded,
    RetainPending,
    Failed(String),
}

fn apply_relay_decision(
    runtime_processing: &mut crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    batch: &mut RelayBatchOutcome,
    relay_p2p_context: Option<&RelayP2pContext>,
    relay_route_map: &RelayRouteMap,
    relay_signing_private_key_hex: Option<&str>,
    relay_nonce_counter: &mut u64,
    service_api_signature_state_hash: &str,
    relay_entry: RelayEntry,
) -> Result<(), ConfigError> {
    let decision = forward_single_entry(
        relay_p2p_context,
        relay_route_map,
        relay_signing_private_key_hex,
        relay_nonce_counter,
        service_api_signature_state_hash,
        &relay_entry,
    )?;
    record_relay_decision(runtime_processing, batch, relay_entry, decision)?;
    Ok(())
}

fn record_relay_decision(
    runtime_processing: &mut crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    batch: &mut RelayBatchOutcome,
    relay_entry: RelayEntry,
    decision: RelayForwardDecision,
) -> Result<(), ConfigError> {
    match decision {
        RelayForwardDecision::Forwarded => {
            batch.relay_message_ids.push(relay_entry.message_id.clone())
        }
        RelayForwardDecision::RetainPending => batch.failed_entries.push(relay_entry),
        RelayForwardDecision::Failed(error_message) => {
            record_forward_failure(runtime_processing, &relay_entry, &error_message)?;
            batch.failed_entries.push(relay_entry);
        }
    }
    Ok(())
}

fn forward_via_http(
    relay_route_map: &RelayRouteMap,
    relay_entry: &RelayEntry,
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
