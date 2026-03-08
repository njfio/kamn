use super::{LiveTransportKamnClient, config::EVENTS_READ_SCOPE, state::build_auth};
use crate::{SdkError, ServiceEventSnapshot};

pub(super) fn read_service_event(
    client: &LiveTransportKamnClient,
) -> Result<ServiceEventSnapshot, SdkError> {
    let auth = build_auth(
        &client.state,
        &client.config,
        &client.config.requester_did,
        "",
        Some(EVENTS_READ_SCOPE),
    )?;
    Ok(client.service_client.read_event_once(&auth)?.into())
}
