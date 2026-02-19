use super::*;

pub(super) fn validate_websocket_route_requirements(
    is_websocket_route: bool,
    headers: &BTreeMap<String, String>,
) -> Result<(), ServiceApiReasonedError> {
    if !is_websocket_route {
        return Ok(());
    }
    validate_websocket_upgrade_headers(headers)
}

pub(super) fn validate_websocket_upgrade_headers(
    headers: &BTreeMap<String, String>,
) -> Result<(), ServiceApiReasonedError> {
    let upgrade = super::header_value(headers, "upgrade").ok_or_else(|| {
        ServiceApiReasonedError::new(
            REASON_CODE_WS_UPGRADE_HEADER_MISSING,
            "missing required websocket upgrade header",
        )
    })?;
    let connection = super::header_value(headers, "connection").ok_or_else(|| {
        ServiceApiReasonedError::new(
            REASON_CODE_WS_CONNECTION_HEADER_MISSING,
            "missing required websocket connection header",
        )
    })?;
    let websocket_key = super::header_value(headers, "sec-websocket-key").ok_or_else(|| {
        ServiceApiReasonedError::new(
            REASON_CODE_WS_KEY_HEADER_MISSING,
            "missing required websocket key header",
        )
    })?;
    let websocket_version =
        super::header_value(headers, "sec-websocket-version").ok_or_else(|| {
            ServiceApiReasonedError::new(
                REASON_CODE_WS_VERSION_HEADER_MISSING,
                "missing required websocket version header",
            )
        })?;

    if !upgrade.eq_ignore_ascii_case("websocket") {
        return Err(ServiceApiReasonedError::new(
            REASON_CODE_WS_UPGRADE_HEADER_INVALID,
            "invalid websocket upgrade header",
        ));
    }
    if !connection.to_ascii_lowercase().contains("upgrade") {
        return Err(ServiceApiReasonedError::new(
            REASON_CODE_WS_CONNECTION_HEADER_INVALID,
            "invalid websocket connection header",
        ));
    }
    if websocket_key.trim().is_empty() {
        return Err(ServiceApiReasonedError::new(
            REASON_CODE_WS_KEY_HEADER_EMPTY,
            "websocket key header must not be empty",
        ));
    }
    if websocket_version.trim() != "13" {
        return Err(ServiceApiReasonedError::new(
            REASON_CODE_WS_VERSION_HEADER_INVALID,
            "invalid websocket version header",
        ));
    }
    Ok(())
}

pub(super) fn websocket_upgrade_response(
    upgrade: WebSocketUpgrade,
    snapshot: ServiceApiSnapshot,
) -> Response {
    let mut response = upgrade
        .on_upgrade(move |socket| stream_websocket_event(socket, snapshot))
        .into_response();
    response
        .headers_mut()
        .insert("X-KAMN-WebSocket-Contract", HeaderValue::from_static("v1"));
    response
}

pub(super) async fn stream_websocket_event(mut socket: WebSocket, snapshot: ServiceApiSnapshot) {
    let payload = ServiceApiWebsocketStateTransitionBody {
        event: "state-transition".to_owned(),
        runtime_mode: snapshot.runtime_mode,
        role: snapshot.role,
        sequence: 1,
    };
    let event_payload = super::serialize_service_api_json(&payload);
    let _ = socket.send(Message::Text(event_payload.into())).await;
}
