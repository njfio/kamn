use super::LiveTransportKamnClient;
use crate::{SdkError, ServiceHealthSnapshot};

pub(super) fn service_health(
    client: &LiveTransportKamnClient,
) -> Result<ServiceHealthSnapshot, SdkError> {
    Ok(client.service_client.health()?.into())
}

pub(super) fn service_metrics(client: &LiveTransportKamnClient) -> Result<String, SdkError> {
    client.service_client.metrics()
}
