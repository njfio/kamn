use super::super::super::*;

pub(super) fn drain_relay_entries(
    service_api_relay_spool_file: Option<&str>,
) -> Result<Vec<crate::service_api_endpoint::ServiceApiRelaySpoolEntry>, ConfigError> {
    crate::service_api_endpoint::drain_service_api_relay_spool_entries(service_api_relay_spool_file)
        .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))
}

pub(super) fn project_relayed_state(
    runtime_processing: &mut crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    service_api_state_file: Option<&str>,
    relay_message_ids: &[String],
) -> Result<(), ConfigError> {
    let projected_count =
        crate::service_api_endpoint::project_service_api_relayed_message_statuses(
            service_api_state_file,
            relay_message_ids,
        )
        .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;
    runtime_processing.relay_projected_state_count = runtime_processing
        .relay_projected_state_count
        .saturating_add(projected_count as u64);
    Ok(())
}

pub(super) fn requeue_failed_entries(
    service_api_relay_spool_file: Option<&str>,
    failed_entries: Vec<crate::service_api_endpoint::ServiceApiRelaySpoolEntry>,
) -> Result<(), ConfigError> {
    for relay_entry in failed_entries {
        crate::service_api_endpoint::append_service_api_relay_spool_entry(
            service_api_relay_spool_file,
            &relay_entry,
        )
        .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;
    }
    Ok(())
}
