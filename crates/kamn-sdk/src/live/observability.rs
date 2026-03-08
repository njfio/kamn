use super::LiveTransportKamnClient;
use crate::{SdkError, ServiceHealthSnapshot};

pub(super) fn service_health(
    client: &LiveTransportKamnClient,
) -> Result<ServiceHealthSnapshot, SdkError> {
    let health = client.service_client.health()?;
    Ok(ServiceHealthSnapshot {
        status: health.status,
        runtime_mode: health.runtime_mode,
        role: health.role,
        observability_source: health.observability_source,
        observability_health: health.observability_health,
    })
}

pub(super) fn service_metrics(client: &LiveTransportKamnClient) -> Result<String, SdkError> {
    client.service_client.metrics()
}
