//! M9 realtime gateway bridge projection contracts.
//!
//! This module projects M9 registry outcomes into deterministic gateway-ready
//! envelopes while preserving fail-closed reason-code taxonomy.

use crate::{
    AntiSpamEngine, ChannelStore, DataLayerM9DispatchAckStatus, DataLayerM9DispatchRequest,
    DataLayerM9PresenceConnectRequest, DataLayerM9PresenceQuery, DataLayerM9RealtimeDeliveryError,
    DataLayerM9RealtimeDeliveryRegistry,
};
use std::fmt;

/// Stable reason marker for unsupported realtime transport profiles.
pub const DATA_LAYER_M9_GATEWAY_UNSUPPORTED_TRANSPORT_REASON_CODE: &str =
    "m9_gateway_unsupported_transport";
/// Stable reason marker emitted when queried presence is visible.
pub const DATA_LAYER_M9_GATEWAY_PRESENCE_VISIBLE_REASON_CODE: &str = "m9_gateway_presence_visible";
/// Stable reason marker emitted when queried presence does not exist.
pub const DATA_LAYER_M9_GATEWAY_PRESENCE_NOT_FOUND_REASON_CODE: &str =
    "m9_gateway_presence_not_found";
/// Stable dispatch event label emitted by gateway projections.
pub const DATA_LAYER_M9_GATEWAY_DISPATCH_EVENT_LABEL: &str = "m9.dispatch.ack";
/// Stable presence event label emitted by gateway projections.
pub const DATA_LAYER_M9_GATEWAY_PRESENCE_EVENT_LABEL: &str = "m9.presence.snapshot";

/// Realtime transport profiles currently supported by the gateway bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM9GatewayTransportProfile {
    /// WebSocket transport profile.
    Websocket,
    /// Server-sent events transport profile.
    ServerSentEvents,
}

impl DataLayerM9GatewayTransportProfile {
    /// Stable transport label for wire/log output.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Websocket => "websocket",
            Self::ServerSentEvents => "sse",
        }
    }
}

/// Dispatch projection request for one M9 event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM9GatewayDispatchProjectionRequest {
    /// Realtime channel identifier used for membership admission.
    pub channel_id: String,
    /// M9 dispatch request.
    pub dispatch_request: DataLayerM9DispatchRequest,
    /// Requested gateway transport profile label.
    pub transport_profile: String,
}

/// Presence projection request combining connect + query phases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM9GatewayPresenceProjectionRequest {
    /// M9 presence connect request.
    pub connect_request: DataLayerM9PresenceConnectRequest,
    /// M9 scoped presence query request.
    pub query: DataLayerM9PresenceQuery,
    /// Requested gateway transport profile label.
    pub transport_profile: String,
}

/// Deterministic gateway dispatch envelope projected from one M9 dispatch outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM9GatewayDispatchProjection {
    /// Stable event label.
    pub event: &'static str,
    /// Normalized transport profile.
    pub transport_profile: DataLayerM9GatewayTransportProfile,
    /// Channel identifier evaluated by dispatch controls.
    pub channel_id: String,
    /// Sender agent DID.
    pub sender_agent_did: String,
    /// Recipient agent DID.
    pub recipient_agent_did: String,
    /// M9 acknowledgement status.
    pub ack_status: DataLayerM9DispatchAckStatus,
    /// Pending queue depth after evaluation.
    pub pending_queue_depth: usize,
    /// Deferred queue depth after evaluation.
    pub deferred_count: usize,
    /// Stable reason marker from M9 dispatch outcome.
    pub reason_code: &'static str,
    /// Backpressure warning marker.
    pub backpressure_warning_event: bool,
    /// Sustained backpressure escrow-extension marker.
    pub escrow_timeout_extension_recommended: bool,
}

/// Deterministic gateway presence envelope projected from connect/query flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM9GatewayPresenceProjection {
    /// Stable event label.
    pub event: &'static str,
    /// Normalized transport profile.
    pub transport_profile: DataLayerM9GatewayTransportProfile,
    /// Requester owner DID.
    pub requester_owner_did: String,
    /// Requester agent DID.
    pub requester_agent_did: String,
    /// Target owner DID.
    pub target_owner_did: String,
    /// Target agent DID.
    pub target_agent_did: String,
    /// True when target presence is visible.
    pub visible: bool,
    /// Gateway node for target when visible.
    pub target_gateway_node: Option<String>,
    /// Last heartbeat timestamp when visible.
    pub target_last_heartbeat_epoch_seconds: Option<u64>,
    /// Stable reason marker for visibility outcome.
    pub reason_code: &'static str,
    /// Deterministic audit tag for gateway presence projection.
    pub audit_record_tag: String,
}

/// Gateway bridge error taxonomy for M9 projection boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM9GatewayBridgeError {
    /// Unsupported transport profile was requested.
    UnsupportedTransport {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Original profile input.
        transport_profile: String,
    },
    /// M9 domain contract rejected the request.
    RealtimeContract(DataLayerM9RealtimeDeliveryError),
}

impl fmt::Display for DataLayerM9GatewayBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedTransport {
                reason_code,
                transport_profile,
            } => write!(
                f,
                "unsupported gateway transport profile: {reason_code} ({transport_profile})"
            ),
            Self::RealtimeContract(error) => {
                write!(f, "m9 realtime contract rejected request: {error}")
            }
        }
    }
}

impl std::error::Error for DataLayerM9GatewayBridgeError {}

/// Projects one M9 dispatch flow into a deterministic gateway envelope.
pub fn data_layer_m9_gateway_project_dispatch_event(
    registry: &mut DataLayerM9RealtimeDeliveryRegistry,
    channel_store: &ChannelStore,
    anti_spam_engine: &mut AntiSpamEngine,
    request: DataLayerM9GatewayDispatchProjectionRequest,
) -> Result<DataLayerM9GatewayDispatchProjection, DataLayerM9GatewayBridgeError> {
    let transport_profile = normalize_transport_profile(request.transport_profile.as_str())?;
    let sender_agent_did = request.dispatch_request.sender_agent_did.clone();
    let recipient_agent_did = request.dispatch_request.recipient_agent_did.clone();
    let channel_id = request.channel_id;
    let outcome = registry
        .dispatch_message_with_controls(
            channel_store,
            anti_spam_engine,
            channel_id.as_str(),
            request.dispatch_request,
        )
        .map_err(DataLayerM9GatewayBridgeError::RealtimeContract)?;
    Ok(DataLayerM9GatewayDispatchProjection {
        event: DATA_LAYER_M9_GATEWAY_DISPATCH_EVENT_LABEL,
        transport_profile,
        channel_id,
        sender_agent_did,
        recipient_agent_did,
        ack_status: outcome.ack_status,
        pending_queue_depth: outcome.pending_queue_depth,
        deferred_count: outcome.deferred_count,
        reason_code: outcome.reason_code,
        backpressure_warning_event: outcome.backpressure_warning_event,
        escrow_timeout_extension_recommended: outcome.escrow_timeout_extension_recommended,
    })
}

/// Projects M9 presence connect/query flows into a deterministic gateway envelope.
pub fn data_layer_m9_gateway_project_presence_event(
    registry: &mut DataLayerM9RealtimeDeliveryRegistry,
    request: DataLayerM9GatewayPresenceProjectionRequest,
) -> Result<DataLayerM9GatewayPresenceProjection, DataLayerM9GatewayBridgeError> {
    let transport_profile = normalize_transport_profile(request.transport_profile.as_str())?;
    let audit_record_tag = build_presence_audit_tag(&request.connect_request, &request.query);

    registry
        .connect_presence(request.connect_request)
        .map_err(DataLayerM9GatewayBridgeError::RealtimeContract)?;
    let presence = registry
        .query_presence(request.query.clone())
        .map_err(DataLayerM9GatewayBridgeError::RealtimeContract)?;

    let (visible, target_gateway_node, target_last_heartbeat_epoch_seconds, reason_code) =
        match presence {
            Some(record) => (
                true,
                Some(record.gateway_node),
                Some(record.last_heartbeat_epoch_seconds),
                DATA_LAYER_M9_GATEWAY_PRESENCE_VISIBLE_REASON_CODE,
            ),
            None => (
                false,
                None,
                None,
                DATA_LAYER_M9_GATEWAY_PRESENCE_NOT_FOUND_REASON_CODE,
            ),
        };

    Ok(DataLayerM9GatewayPresenceProjection {
        event: DATA_LAYER_M9_GATEWAY_PRESENCE_EVENT_LABEL,
        transport_profile,
        requester_owner_did: request.query.requester_owner_did,
        requester_agent_did: request.query.requester_agent_did,
        target_owner_did: request.query.owner_did,
        target_agent_did: request.query.target_agent_did,
        visible,
        target_gateway_node,
        target_last_heartbeat_epoch_seconds,
        reason_code,
        audit_record_tag,
    })
}

fn normalize_transport_profile(
    value: &str,
) -> Result<DataLayerM9GatewayTransportProfile, DataLayerM9GatewayBridgeError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "websocket" | "ws" => Ok(DataLayerM9GatewayTransportProfile::Websocket),
        "sse" | "server-sent-events" => Ok(DataLayerM9GatewayTransportProfile::ServerSentEvents),
        _ => Err(DataLayerM9GatewayBridgeError::UnsupportedTransport {
            reason_code: DATA_LAYER_M9_GATEWAY_UNSUPPORTED_TRANSPORT_REASON_CODE,
            transport_profile: value.to_owned(),
        }),
    }
}

fn build_presence_audit_tag(
    connect_request: &DataLayerM9PresenceConnectRequest,
    query: &DataLayerM9PresenceQuery,
) -> String {
    format!(
        "m9_presence:{}:{}:{}:{}:{}",
        connect_request.owner_did,
        connect_request.agent_did,
        query.requester_agent_did,
        query.target_agent_did,
        connect_request.last_heartbeat_epoch_seconds
    )
}
