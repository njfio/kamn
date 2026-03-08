use super::{LiveTransportKamnClient, config::EVENTS_READ_SCOPE, state::build_auth};
use crate::{SdkError, ServiceEventSnapshot, ServiceRequestAuth};

pub(super) fn read_service_event(
    client: &LiveTransportKamnClient,
) -> Result<ServiceEventSnapshot, SdkError> {
    let auth = events_read_auth(client)?;
    Ok(client.service_client.read_event_once(&auth)?.into())
}

fn events_read_auth(client: &LiveTransportKamnClient) -> Result<ServiceRequestAuth, SdkError> {
    build_auth(
        &client.state,
        &client.config,
        &client.config.requester_did,
        "",
        Some(EVENTS_READ_SCOPE),
    )
}
