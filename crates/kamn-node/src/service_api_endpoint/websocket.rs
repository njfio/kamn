use super::*;

const WS_EVENTS_MODE_STATE_TRANSITION: &str = "state-transition";
const WS_EVENTS_MODE_PRESENCE: &str = "presence";
const WS_PRESENCE_DEFAULT_GATEWAY_NODE: &str = "service-api-gateway";
const WS_PRESENCE_DEFAULT_CONNECTED_SINCE_EPOCH_SECONDS: u64 = 1_709_000_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ServiceApiWebsocketPresenceBody {
    event: &'static str,
    transport_profile: &'static str,
    requester_owner_did: String,
    requester_agent_did: String,
    target_owner_did: String,
    target_agent_did: String,
    visible: bool,
    target_gateway_node: Option<String>,
    target_last_heartbeat_epoch_seconds: Option<u64>,
    reason_code: &'static str,
    audit_record_tag: String,
    sequence: u64,
}

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

pub(super) fn project_websocket_event_payload(
    snapshot: &ServiceApiSnapshot,
    headers: &BTreeMap<String, String>,
) -> Result<String, ServiceApiReasonedError> {
    let mode = super::header_value(headers, REQUEST_WS_EVENTS_MODE_HEADER)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(WS_EVENTS_MODE_STATE_TRANSITION);
    if mode.eq_ignore_ascii_case(WS_EVENTS_MODE_STATE_TRANSITION) {
        return Ok(serialize_state_transition_payload(snapshot));
    }
    if !mode.eq_ignore_ascii_case(WS_EVENTS_MODE_PRESENCE) {
        return Err(ServiceApiReasonedError::new(
            REASON_CODE_WS_EVENTS_MODE_INVALID,
            format!(
                "invalid websocket events mode header: {}={mode}",
                REQUEST_WS_EVENTS_MODE_HEADER
            ),
        ));
    }
    project_presence_mode_payload(headers)
}

pub(super) fn project_websocket_error_response(
    error: &ServiceApiReasonedError,
) -> (StatusCode, &'static str, &'static str) {
    if matches!(
        error.reason_code,
        DATA_LAYER_M9_OWNER_SCOPE_DENIED_REASON_CODE
            | DATA_LAYER_M9_PRESENCE_VISIBILITY_DENIED_REASON_CODE
    ) {
        return (StatusCode::FORBIDDEN, "forbidden", "websocket-forbidden");
    }
    (
        StatusCode::BAD_REQUEST,
        "bad-request",
        "websocket-bad-request",
    )
}

pub(super) fn websocket_upgrade_response(
    upgrade: WebSocketUpgrade,
    event_payload: String,
) -> Response {
    let mut response = upgrade
        .on_upgrade(move |socket| stream_websocket_event(socket, event_payload))
        .into_response();
    response
        .headers_mut()
        .insert("X-KAMN-WebSocket-Contract", HeaderValue::from_static("v1"));
    response
}

pub(super) async fn stream_websocket_event(mut socket: WebSocket, event_payload: String) {
    let _ = socket.send(Message::Text(event_payload.into())).await;
}

fn serialize_state_transition_payload(snapshot: &ServiceApiSnapshot) -> String {
    let payload = ServiceApiWebsocketStateTransitionBody {
        event: WS_EVENTS_MODE_STATE_TRANSITION.to_owned(),
        runtime_mode: snapshot.runtime_mode.clone(),
        role: snapshot.role.clone(),
        sequence: 1,
    };
    super::serialize_service_api_json(&payload)
}

fn project_presence_mode_payload(
    headers: &BTreeMap<String, String>,
) -> Result<String, ServiceApiReasonedError> {
    let requester_agent_did = required_presence_header(
        headers,
        REQUEST_AUTH_SENDER_DID_HEADER,
        REASON_CODE_WS_PRESENCE_REQUESTER_AGENT_DID_HEADER_MISSING,
    )?
    .to_owned();
    let owner_did = required_presence_header(
        headers,
        REQUEST_WS_PRESENCE_OWNER_DID_HEADER,
        REASON_CODE_WS_PRESENCE_OWNER_DID_HEADER_MISSING,
    )?
    .to_owned();
    let target_owner_did =
        super::header_value(headers, REQUEST_WS_PRESENCE_TARGET_OWNER_DID_HEADER)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(owner_did.as_str())
            .to_owned();
    let target_agent_did = required_presence_header(
        headers,
        REQUEST_WS_PRESENCE_TARGET_AGENT_DID_HEADER,
        REASON_CODE_WS_PRESENCE_TARGET_AGENT_DID_HEADER_MISSING,
    )?
    .to_owned();
    let gateway_node = super::header_value(headers, REQUEST_WS_PRESENCE_GATEWAY_NODE_HEADER)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(WS_PRESENCE_DEFAULT_GATEWAY_NODE)
        .to_owned();
    let connected_since_epoch_seconds = parse_presence_timestamp_header(
        headers,
        REQUEST_WS_PRESENCE_CONNECTED_SINCE_HEADER,
        REASON_CODE_WS_PRESENCE_CONNECTED_SINCE_INVALID,
        WS_PRESENCE_DEFAULT_CONNECTED_SINCE_EPOCH_SECONDS,
    )?;
    let last_heartbeat_epoch_seconds = parse_presence_timestamp_header(
        headers,
        REQUEST_WS_PRESENCE_LAST_HEARTBEAT_HEADER,
        REASON_CODE_WS_PRESENCE_LAST_HEARTBEAT_INVALID,
        connected_since_epoch_seconds,
    )?;
    let capabilities_active = parse_presence_capabilities(headers)?;

    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let projection = data_layer_m9_gateway_project_presence_event(
        &mut registry,
        DataLayerM9GatewayPresenceProjectionRequest {
            connect_request: DataLayerM9PresenceConnectRequest {
                requester_owner_did: owner_did.clone(),
                owner_did: owner_did.clone(),
                agent_did: target_agent_did.clone(),
                connected_since_epoch_seconds,
                last_heartbeat_epoch_seconds,
                gateway_node,
                capabilities_active,
            },
            query: DataLayerM9PresenceQuery {
                requester_owner_did: owner_did.clone(),
                owner_did: target_owner_did,
                requester_agent_did: requester_agent_did.clone(),
                target_agent_did: target_agent_did.clone(),
            },
            transport_profile: "websocket".to_owned(),
        },
    )
    .map_err(map_presence_projection_error)?;

    let payload = ServiceApiWebsocketPresenceBody {
        event: projection.event,
        transport_profile: projection.transport_profile.as_str(),
        requester_owner_did: projection.requester_owner_did,
        requester_agent_did: projection.requester_agent_did,
        target_owner_did: projection.target_owner_did,
        target_agent_did: projection.target_agent_did,
        visible: projection.visible,
        target_gateway_node: projection.target_gateway_node,
        target_last_heartbeat_epoch_seconds: projection.target_last_heartbeat_epoch_seconds,
        reason_code: projection.reason_code,
        audit_record_tag: projection.audit_record_tag,
        sequence: 1,
    };
    Ok(super::serialize_service_api_json(&payload))
}

fn required_presence_header<'a>(
    headers: &'a BTreeMap<String, String>,
    header_name: &str,
    reason_code: &'static str,
) -> Result<&'a str, ServiceApiReasonedError> {
    super::header_value(headers, header_name)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ServiceApiReasonedError::new(
                reason_code,
                format!("missing required header: {header_name}"),
            )
        })
}

fn parse_presence_timestamp_header(
    headers: &BTreeMap<String, String>,
    header_name: &str,
    reason_code: &'static str,
    default_value: u64,
) -> Result<u64, ServiceApiReasonedError> {
    let Some(raw_value) = super::header_value(headers, header_name) else {
        return Ok(default_value);
    };
    raw_value.trim().parse::<u64>().map_err(|_| {
        ServiceApiReasonedError::new(
            reason_code,
            format!("invalid u64 header value: {header_name}"),
        )
    })
}

fn parse_presence_capabilities(
    headers: &BTreeMap<String, String>,
) -> Result<Vec<String>, ServiceApiReasonedError> {
    let Some(raw_capabilities) =
        super::header_value(headers, REQUEST_WS_PRESENCE_CAPABILITIES_HEADER)
    else {
        return Ok(vec!["ws".to_owned()]);
    };
    let capabilities = raw_capabilities
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if capabilities.is_empty() {
        return Err(ServiceApiReasonedError::new(
            REASON_CODE_WS_PRESENCE_CAPABILITIES_INVALID,
            format!(
                "presence capabilities header must include at least one capability: {REQUEST_WS_PRESENCE_CAPABILITIES_HEADER}"
            ),
        ));
    }
    Ok(capabilities)
}

fn map_presence_projection_error(error: DataLayerM9GatewayBridgeError) -> ServiceApiReasonedError {
    match error {
        DataLayerM9GatewayBridgeError::UnsupportedTransport {
            reason_code,
            transport_profile,
        } => ServiceApiReasonedError::new(
            reason_code,
            format!("unsupported websocket presence transport profile: {transport_profile}"),
        ),
        DataLayerM9GatewayBridgeError::RealtimeContract(contract_error) => {
            map_realtime_contract_error(contract_error)
        }
    }
}

fn map_realtime_contract_error(error: DataLayerM9RealtimeDeliveryError) -> ServiceApiReasonedError {
    match error {
        DataLayerM9RealtimeDeliveryError::InvalidDid {
            field,
            reason_code,
            detail,
        } => ServiceApiReasonedError::new(
            reason_code,
            format!("presence projection invalid did field {field}: {detail}"),
        ),
        DataLayerM9RealtimeDeliveryError::OwnerScopeViolation { reason_code } => {
            ServiceApiReasonedError::new(reason_code, "presence projection owner scope denied")
        }
        DataLayerM9RealtimeDeliveryError::PresenceVisibilityDenied { reason_code } => {
            ServiceApiReasonedError::new(reason_code, "presence projection visibility denied")
        }
        DataLayerM9RealtimeDeliveryError::InvalidTimestampOrder {
            connected_since_epoch_seconds,
            last_heartbeat_epoch_seconds,
        } => ServiceApiReasonedError::new(
            REASON_CODE_WS_PRESENCE_LAST_HEARTBEAT_INVALID,
            format!(
                "invalid timestamp order: connected_since={connected_since_epoch_seconds}, last_heartbeat={last_heartbeat_epoch_seconds}"
            ),
        ),
        DataLayerM9RealtimeDeliveryError::EmptyField(field) => ServiceApiReasonedError::new(
            REASON_CODE_WS_PRESENCE_PROJECTION_INVALID,
            format!("presence projection field must not be empty: {field}"),
        ),
        other => ServiceApiReasonedError::new(
            REASON_CODE_WS_PRESENCE_PROJECTION_INVALID,
            format!("presence projection failed: {other}"),
        ),
    }
}
