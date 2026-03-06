use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::broadcast;

const WS_EVENTS_MODE_STATE_TRANSITION: &str = "state-transition";
const WS_EVENTS_MODE_PRESENCE: &str = "presence";
const WS_PRESENCE_DEFAULT_GATEWAY_NODE: &str = "service-api-gateway";
const WS_PRESENCE_DEFAULT_CONNECTED_SINCE_EPOCH_SECONDS: u64 = 1_709_000_000;
const WS_EVENT_BUFFER_CAPACITY: usize = 256;
const WS_EVENT_MESSAGE_CREATED: &str = "service-api.message.created";
const WS_EVENT_CHANNEL_CREATED: &str = "service-api.channel.created";
const WS_EVENT_TASK_SUBMITTED: &str = "service-api.task.submitted";
const WS_EVENT_TASK_ACCEPTED: &str = "service-api.task.accepted";
const WS_EVENT_TASK_COMPLETED: &str = "service-api.task.completed";
const WS_EVENT_BRIDGE_SUBMITTED: &str = "service-api.bridge.submitted";
const WS_EVENT_BRIDGE_FORWARDED: &str = "service-api.bridge.forwarded";

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ServiceApiWebsocketMessageLifecycleBody {
    event: &'static str,
    message_id: String,
    status: String,
    runtime_mode: String,
    sender_did: Option<String>,
    recipient_did: Option<String>,
    channel_id: Option<String>,
    sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ServiceApiWebsocketChannelLifecycleBody {
    event: &'static str,
    channel_id: String,
    status: String,
    sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ServiceApiWebsocketTaskLifecycleBody {
    event: &'static str,
    task_id: String,
    state: String,
    sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ServiceApiWebsocketBridgeLifecycleBody {
    event: &'static str,
    bridge_id: String,
    bridge_status: String,
    source_message_id: Option<String>,
    target_message_id: Option<String>,
    forward_tx_hash: Option<String>,
    sequence: u64,
}

#[derive(Debug)]
pub(super) struct ServiceApiWebsocketEventFanout {
    sender: broadcast::Sender<String>,
    sequence: AtomicU64,
}

impl ServiceApiWebsocketEventFanout {
    pub(super) fn new() -> Self {
        let (sender, _) = broadcast::channel(WS_EVENT_BUFFER_CAPACITY);
        Self {
            sender,
            sequence: AtomicU64::new(0),
        }
    }

    fn next_sequence(&self) -> u64 {
        self.sequence
            .fetch_add(1, Ordering::SeqCst)
            .saturating_add(1)
    }

    pub(super) fn subscribe(&self) -> broadcast::Receiver<String> {
        self.sender.subscribe()
    }

    pub(super) fn publish_message_created_event(
        &self,
        payload: &ServiceApiMessageCreateBody,
        sender_did: Option<&str>,
        recipient_did: Option<&str>,
        channel_id: Option<&str>,
    ) {
        let event = ServiceApiWebsocketMessageLifecycleBody {
            event: WS_EVENT_MESSAGE_CREATED,
            message_id: payload.message_id.clone(),
            status: payload.status.clone(),
            runtime_mode: payload.runtime_mode.clone(),
            sender_did: sender_did.map(str::to_owned),
            recipient_did: recipient_did.map(str::to_owned),
            channel_id: channel_id.map(str::to_owned),
            sequence: self.next_sequence(),
        };
        let event_payload = super::serialize_service_api_json(&event);
        let _ = self.sender.send(event_payload);
    }

    pub(super) fn publish_channel_created_event(&self, payload: &ServiceApiChannelCreateBody) {
        let event = ServiceApiWebsocketChannelLifecycleBody {
            event: WS_EVENT_CHANNEL_CREATED,
            channel_id: payload.channel_id.clone(),
            status: payload.status.clone(),
            sequence: self.next_sequence(),
        };
        let event_payload = super::serialize_service_api_json(&event);
        let _ = self.sender.send(event_payload);
    }

    pub(super) fn publish_task_submitted_event(&self, payload: &ServiceApiTaskCreateBody) {
        let event = ServiceApiWebsocketTaskLifecycleBody {
            event: WS_EVENT_TASK_SUBMITTED,
            task_id: payload.task_id.clone(),
            state: payload.state.clone(),
            sequence: self.next_sequence(),
        };
        let event_payload = super::serialize_service_api_json(&event);
        let _ = self.sender.send(event_payload);
    }

    pub(super) fn publish_task_transition_event(&self, payload: &ServiceApiTaskTransitionBody) {
        let event_name = if payload.state == "accepted" {
            WS_EVENT_TASK_ACCEPTED
        } else if payload.state == "completed" {
            WS_EVENT_TASK_COMPLETED
        } else {
            WS_EVENT_TASK_SUBMITTED
        };
        let event = ServiceApiWebsocketTaskLifecycleBody {
            event: event_name,
            task_id: payload.task_id.clone(),
            state: payload.state.clone(),
            sequence: self.next_sequence(),
        };
        let event_payload = super::serialize_service_api_json(&event);
        let _ = self.sender.send(event_payload);
    }

    pub(super) fn publish_bridge_submitted_event(&self, payload: &ServiceApiBridgeSubmitBody) {
        let event = ServiceApiWebsocketBridgeLifecycleBody {
            event: WS_EVENT_BRIDGE_SUBMITTED,
            bridge_id: payload.bridge_id.clone(),
            bridge_status: payload.bridge_status.clone(),
            source_message_id: Some(payload.source_message_id.clone()),
            target_message_id: None,
            forward_tx_hash: None,
            sequence: self.next_sequence(),
        };
        let event_payload = super::serialize_service_api_json(&event);
        let _ = self.sender.send(event_payload);
    }

    pub(super) fn publish_bridge_forwarded_event(&self, payload: &ServiceApiBridgeStatusBody) {
        let event = ServiceApiWebsocketBridgeLifecycleBody {
            event: WS_EVENT_BRIDGE_FORWARDED,
            bridge_id: payload.bridge_id.clone(),
            bridge_status: payload.bridge_status.clone(),
            source_message_id: None,
            target_message_id: Some(payload.target_message_id.clone()),
            forward_tx_hash: Some(payload.forward_tx_hash.clone()),
            sequence: self.next_sequence(),
        };
        let event_payload = super::serialize_service_api_json(&event);
        let _ = self.sender.send(event_payload);
    }
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
    let upgrade = super::auth::header_value(headers, "upgrade").ok_or_else(|| {
        ServiceApiReasonedError::new(
            REASON_CODE_WS_UPGRADE_HEADER_MISSING,
            "missing required websocket upgrade header",
        )
    })?;
    let connection = super::auth::header_value(headers, "connection").ok_or_else(|| {
        ServiceApiReasonedError::new(
            REASON_CODE_WS_CONNECTION_HEADER_MISSING,
            "missing required websocket connection header",
        )
    })?;
    let websocket_key =
        super::auth::header_value(headers, "sec-websocket-key").ok_or_else(|| {
            ServiceApiReasonedError::new(
                REASON_CODE_WS_KEY_HEADER_MISSING,
                "missing required websocket key header",
            )
        })?;
    let websocket_version = super::auth::header_value(headers, "sec-websocket-version")
        .ok_or_else(|| {
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
    let mode = super::auth::header_value(headers, REQUEST_WS_EVENTS_MODE_HEADER)
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
    websocket_events: &ServiceApiWebsocketEventFanout,
) -> Response {
    let websocket_events = websocket_events.subscribe();
    let mut response = upgrade
        .on_upgrade(move |socket| stream_websocket_event(socket, event_payload, websocket_events))
        .into_response();
    response
        .headers_mut()
        .insert("X-KAMN-WebSocket-Contract", HeaderValue::from_static("v1"));
    response
}

pub(super) async fn stream_websocket_event(
    mut socket: WebSocket,
    event_payload: String,
    mut websocket_events: broadcast::Receiver<String>,
) {
    if socket
        .send(Message::Text(event_payload.into()))
        .await
        .is_err()
    {
        return;
    }

    loop {
        tokio::select! {
            result = websocket_events.recv() => {
                match result {
                    Ok(event_payload) => {
                        if socket.send(Message::Text(event_payload.into())).await.is_err() {
                            return;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => return,
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Close(_))) | None => return,
                    Some(Ok(Message::Ping(payload))) => {
                        if socket.send(Message::Pong(payload)).await.is_err() {
                            return;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(_)) => {}
                    Some(Err(_)) => return,
                }
            }
        }
    }
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
    let requester_agent_did = required_non_empty_presence_header(
        headers,
        REQUEST_AUTH_SENDER_DID_HEADER,
        REASON_CODE_WS_PRESENCE_REQUESTER_AGENT_DID_HEADER_MISSING,
    )?
    .to_owned();
    let owner_did = required_presence_kamn_did_header(
        headers,
        REQUEST_WS_PRESENCE_OWNER_DID_HEADER,
        REASON_CODE_WS_PRESENCE_OWNER_DID_HEADER_MISSING,
        REASON_CODE_WS_PRESENCE_OWNER_DID_HEADER_INVALID,
        "invalid presence owner did header",
    )?
    .to_owned();
    let target_owner_did = optional_presence_kamn_did_header(
        headers,
        REQUEST_WS_PRESENCE_TARGET_OWNER_DID_HEADER,
        owner_did.as_str(),
        REASON_CODE_WS_PRESENCE_TARGET_OWNER_DID_HEADER_INVALID,
        "invalid presence target owner did header",
    )?;
    let target_agent_did = required_presence_agent_did_header(
        headers,
        REQUEST_WS_PRESENCE_TARGET_AGENT_DID_HEADER,
        REASON_CODE_WS_PRESENCE_TARGET_AGENT_DID_HEADER_MISSING,
        REASON_CODE_WS_PRESENCE_TARGET_AGENT_DID_HEADER_INVALID,
        "invalid presence target agent did header",
    )?
    .to_owned();
    let gateway_node = super::auth::header_value(headers, REQUEST_WS_PRESENCE_GATEWAY_NODE_HEADER)
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

fn required_non_empty_presence_header<'a>(
    headers: &'a BTreeMap<String, String>,
    header_name: &str,
    reason_code: &'static str,
) -> Result<&'a str, ServiceApiReasonedError> {
    super::auth::header_value(headers, header_name)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ServiceApiReasonedError::new(
                reason_code,
                format!("missing required header: {header_name}"),
            )
        })
}

fn required_presence_agent_did_header<'a>(
    headers: &'a BTreeMap<String, String>,
    header_name: &str,
    missing_reason_code: &'static str,
    invalid_reason_code: &'static str,
    invalid_message: &str,
) -> Result<&'a str, ServiceApiReasonedError> {
    let header_value =
        required_non_empty_presence_header(headers, header_name, missing_reason_code)?;
    AgentDid::parse(header_value).map_err(|error| {
        ServiceApiReasonedError::new(invalid_reason_code, format!("{invalid_message}: {error}"))
    })?;
    Ok(header_value)
}

fn required_presence_kamn_did_header<'a>(
    headers: &'a BTreeMap<String, String>,
    header_name: &str,
    missing_reason_code: &'static str,
    invalid_reason_code: &'static str,
    invalid_message: &str,
) -> Result<&'a str, ServiceApiReasonedError> {
    let header_value =
        required_non_empty_presence_header(headers, header_name, missing_reason_code)?;
    KamnDid::parse(header_value).map_err(|error| {
        ServiceApiReasonedError::new(invalid_reason_code, format!("{invalid_message}: {error}"))
    })?;
    Ok(header_value)
}

fn optional_presence_kamn_did_header(
    headers: &BTreeMap<String, String>,
    header_name: &str,
    default_value: &str,
    invalid_reason_code: &'static str,
    invalid_message: &str,
) -> Result<String, ServiceApiReasonedError> {
    let Some(header_value) = super::auth::header_value(headers, header_name)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(default_value.to_owned());
    };
    KamnDid::parse(header_value).map_err(|error| {
        ServiceApiReasonedError::new(invalid_reason_code, format!("{invalid_message}: {error}"))
    })?;
    Ok(header_value.to_owned())
}

fn parse_presence_timestamp_header(
    headers: &BTreeMap<String, String>,
    header_name: &str,
    reason_code: &'static str,
    default_value: u64,
) -> Result<u64, ServiceApiReasonedError> {
    let Some(raw_value) = super::auth::header_value(headers, header_name) else {
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
        super::auth::header_value(headers, REQUEST_WS_PRESENCE_CAPABILITIES_HEADER)
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

#[cfg(test)]
mod tests {
    use super::ServiceApiWebsocketEventFanout;
    use crate::service_api_endpoint::{
        ServiceApiBridgeStatusBody, ServiceApiBridgeSubmitBody, ServiceApiChannelCreateBody,
        ServiceApiTaskCreateBody, ServiceApiTaskTransitionBody,
    };
    use serde_json::Value;

    #[test]
    fn regression_issue_6216_websocket_fanout_publishes_channel_task_bridge_lifecycle_events() {
        let fanout = ServiceApiWebsocketEventFanout::new();
        let mut receiver = fanout.subscribe();

        fanout.publish_channel_created_event(&ServiceApiChannelCreateBody {
            channel_id: "channel-1".to_owned(),
            status: "created".to_owned(),
        });
        fanout.publish_task_submitted_event(&ServiceApiTaskCreateBody {
            task_id: "task-1".to_owned(),
            state: "submitted".to_owned(),
        });
        fanout.publish_task_transition_event(&ServiceApiTaskTransitionBody {
            task_id: "task-1".to_owned(),
            state: "accepted".to_owned(),
        });
        fanout.publish_bridge_submitted_event(&ServiceApiBridgeSubmitBody {
            bridge_id: "bridge-1".to_owned(),
            source_message_id: "msg-source-1".to_owned(),
            bridge_status: "submitted".to_owned(),
        });
        fanout.publish_bridge_forwarded_event(&ServiceApiBridgeStatusBody {
            bridge_id: "bridge-1".to_owned(),
            bridge_status: "forwarded".to_owned(),
            target_message_id: "msg-target-1".to_owned(),
            forward_tx_hash: "sha256:bridge-forwarded-1".to_owned(),
        });

        let mut events = Vec::new();
        for _ in 0..5 {
            events.push(
                receiver
                    .try_recv()
                    .expect("fanout should publish lifecycle event"),
            );
        }

        let parsed = events
            .iter()
            .map(|payload| {
                serde_json::from_str::<Value>(payload).expect("event payload should be valid json")
            })
            .collect::<Vec<_>>();

        assert_eq!(
            parsed[0].get("event").and_then(Value::as_str),
            Some("service-api.channel.created")
        );
        assert_eq!(
            parsed[1].get("event").and_then(Value::as_str),
            Some("service-api.task.submitted")
        );
        assert_eq!(
            parsed[2].get("event").and_then(Value::as_str),
            Some("service-api.task.accepted")
        );
        assert_eq!(
            parsed[3].get("event").and_then(Value::as_str),
            Some("service-api.bridge.submitted")
        );
        assert_eq!(
            parsed[4].get("event").and_then(Value::as_str),
            Some("service-api.bridge.forwarded")
        );

        let sequences = parsed
            .iter()
            .map(|payload| payload.get("sequence").and_then(Value::as_u64))
            .collect::<Vec<_>>();
        assert_eq!(sequences, vec![Some(1), Some(2), Some(3), Some(4), Some(5)]);
    }
}
