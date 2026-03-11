use super::*;

pub(super) async fn handle_service_api_websocket_route(
    State(state): State<Arc<ServiceApiRuntimeState>>,
    Extension(context): Extension<ServiceApiRequestContext>,
    upgrade: WebSocketUpgrade,
) -> Response {
    let _ = context.correlation_id.as_str();
    let event_payload = super::websocket::project_websocket_event_payload(
        &state.snapshot,
        &context.parsed_request.headers,
    );
    match event_payload {
        Ok(payload) => upgrade_response(upgrade, payload, &state),
        Err(error) => websocket_error_response(error),
    }
}

fn upgrade_response(
    upgrade: WebSocketUpgrade,
    event_payload: String,
    state: &Arc<ServiceApiRuntimeState>,
) -> Response {
    let mut response = super::websocket::websocket_upgrade_response(
        upgrade,
        event_payload,
        &state.websocket_events,
    );
    response
        .extensions_mut()
        .insert(ServiceApiRequestOutcome("websocket-upgrade"));
    response
}

fn websocket_error_response(error: ServiceApiReasonedError) -> Response {
    let (status_code, error_label, outcome) =
        super::websocket::project_websocket_error_response(&error);
    let mut response = super::payload::json_error_response(
        status_code,
        error_label,
        error.reason_code,
        error.message.as_str(),
    );
    response
        .extensions_mut()
        .insert(ServiceApiRequestOutcome(outcome));
    response
}
