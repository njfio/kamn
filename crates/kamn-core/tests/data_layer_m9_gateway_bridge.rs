use kamn_core::{
    data_layer_m9_gateway_project_dispatch_event, data_layer_m9_gateway_project_presence_event,
    AntiSpamConfig, AntiSpamEngine, ChannelStore, DataLayerM9DispatchAckStatus,
    DataLayerM9DispatchRequest, DataLayerM9GatewayBridgeError,
    DataLayerM9GatewayDispatchProjectionRequest, DataLayerM9GatewayPresenceProjectionRequest,
    DataLayerM9GatewayTransportProfile, DataLayerM9PresenceConnectRequest,
    DataLayerM9PresenceQuery, DataLayerM9RealtimeDeliveryError,
    DataLayerM9RealtimeDeliveryRegistry, DATA_LAYER_M9_ACK_DELIVERED_REASON_CODE,
    DATA_LAYER_M9_ANTI_SPAM_INSUFFICIENT_DEPOSIT_REASON_CODE,
    DATA_LAYER_M9_GATEWAY_DISPATCH_EVENT_LABEL, DATA_LAYER_M9_GATEWAY_PRESENCE_EVENT_LABEL,
    DATA_LAYER_M9_GATEWAY_PRESENCE_VISIBLE_REASON_CODE,
    DATA_LAYER_M9_GATEWAY_UNSUPPORTED_TRANSPORT_REASON_CODE,
    DATA_LAYER_M9_OWNER_SCOPE_DENIED_REASON_CODE,
    DATA_LAYER_M9_PRESENCE_VISIBILITY_DENIED_REASON_CODE,
};

fn dispatch_request(
    message_id: &str,
    dispatched_at_epoch_seconds: u64,
) -> DataLayerM9DispatchRequest {
    DataLayerM9DispatchRequest {
        requester_owner_did: "kamn:did:owner:alpha".to_owned(),
        owner_did: "kamn:did:owner:alpha".to_owned(),
        sender_agent_did: "kamn:did:agent:alpha-sender".to_owned(),
        recipient_agent_did: "kamn:did:agent:alpha-recipient".to_owned(),
        message_id: message_id.to_owned(),
        dispatched_at_epoch_seconds,
    }
}

fn presence_connect(agent_did: &str) -> DataLayerM9PresenceConnectRequest {
    DataLayerM9PresenceConnectRequest {
        requester_owner_did: "kamn:did:owner:alpha".to_owned(),
        owner_did: "kamn:did:owner:alpha".to_owned(),
        agent_did: agent_did.to_owned(),
        connected_since_epoch_seconds: 1_709_000_000,
        last_heartbeat_epoch_seconds: 1_709_000_005,
        gateway_node: "gateway-alpha".to_owned(),
        capabilities_active: vec!["ws".to_owned()],
    }
}

#[test]
fn spec_c01_m9_gateway_dispatch_projection_is_deterministic_for_supported_transport() {
    let mut channel_store = ChannelStore::new();
    channel_store
        .create_direct(
            "m9-gateway-direct",
            "kamn:did:agent:alpha-sender",
            "kamn:did:agent:alpha-recipient",
        )
        .expect("direct channel should be created");

    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    registry
        .connect_presence(presence_connect("kamn:did:agent:alpha-recipient"))
        .expect("recipient presence should be connected");
    let mut anti_spam = AntiSpamEngine::new(AntiSpamConfig::default())
        .expect("default anti-spam config should initialize");
    anti_spam
        .set_deposit("kamn:did:agent:alpha-sender", 100)
        .expect("sender deposit should satisfy anti-spam policy");

    let projection = data_layer_m9_gateway_project_dispatch_event(
        &mut registry,
        &channel_store,
        &mut anti_spam,
        DataLayerM9GatewayDispatchProjectionRequest {
            channel_id: "m9-gateway-direct".to_owned(),
            dispatch_request: dispatch_request("m9-gateway-msg-001", 1_709_000_010),
            transport_profile: "websocket".to_owned(),
        },
    )
    .expect("dispatch projection should succeed");

    assert_eq!(projection.event, DATA_LAYER_M9_GATEWAY_DISPATCH_EVENT_LABEL);
    assert_eq!(
        projection.transport_profile,
        DataLayerM9GatewayTransportProfile::Websocket
    );
    assert_eq!(projection.channel_id, "m9-gateway-direct");
    assert_eq!(projection.sender_agent_did, "kamn:did:agent:alpha-sender");
    assert_eq!(
        projection.recipient_agent_did,
        "kamn:did:agent:alpha-recipient"
    );
    assert_eq!(
        projection.ack_status,
        DataLayerM9DispatchAckStatus::Delivered
    );
    assert_eq!(projection.pending_queue_depth, 0);
    assert_eq!(projection.deferred_count, 0);
    assert_eq!(
        projection.reason_code,
        DATA_LAYER_M9_ACK_DELIVERED_REASON_CODE
    );
    assert!(!projection.backpressure_warning_event);
    assert!(!projection.escrow_timeout_extension_recommended);
}

#[test]
fn spec_c02_m9_gateway_presence_projection_is_deterministic_for_owner_scoped_self_query() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let connect_request = presence_connect("kamn:did:agent:alpha-self");

    let projection = data_layer_m9_gateway_project_presence_event(
        &mut registry,
        DataLayerM9GatewayPresenceProjectionRequest {
            connect_request: connect_request.clone(),
            query: DataLayerM9PresenceQuery {
                requester_owner_did: "kamn:did:owner:alpha".to_owned(),
                owner_did: "kamn:did:owner:alpha".to_owned(),
                requester_agent_did: "kamn:did:agent:alpha-self".to_owned(),
                target_agent_did: "kamn:did:agent:alpha-self".to_owned(),
            },
            transport_profile: "sse".to_owned(),
        },
    )
    .expect("presence projection should succeed");

    assert_eq!(projection.event, DATA_LAYER_M9_GATEWAY_PRESENCE_EVENT_LABEL);
    assert_eq!(
        projection.transport_profile,
        DataLayerM9GatewayTransportProfile::ServerSentEvents
    );
    assert_eq!(projection.requester_owner_did, "kamn:did:owner:alpha");
    assert_eq!(projection.requester_agent_did, "kamn:did:agent:alpha-self");
    assert_eq!(projection.target_owner_did, "kamn:did:owner:alpha");
    assert_eq!(projection.target_agent_did, "kamn:did:agent:alpha-self");
    assert!(projection.visible);
    assert_eq!(
        projection.target_gateway_node.as_deref(),
        Some("gateway-alpha")
    );
    assert_eq!(
        projection.target_last_heartbeat_epoch_seconds,
        Some(1_709_000_005)
    );
    assert_eq!(
        projection.reason_code,
        DATA_LAYER_M9_GATEWAY_PRESENCE_VISIBLE_REASON_CODE
    );
    assert_eq!(
        projection.audit_record_tag,
        "m9_presence:kamn:did:owner:alpha:kamn:did:agent:alpha-self:kamn:did:agent:alpha-self:kamn:did:agent:alpha-self:1709000005"
    );
}

#[test]
fn spec_c03_m9_gateway_projection_fails_closed_for_unsupported_transport() {
    let mut channel_store = ChannelStore::new();
    channel_store
        .create_direct(
            "m9-gateway-unsupported",
            "kamn:did:agent:alpha-sender",
            "kamn:did:agent:alpha-recipient",
        )
        .expect("direct channel should be created");
    let mut anti_spam = AntiSpamEngine::new(AntiSpamConfig::default())
        .expect("default anti-spam config should initialize");
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();

    let unsupported = data_layer_m9_gateway_project_dispatch_event(
        &mut registry,
        &channel_store,
        &mut anti_spam,
        DataLayerM9GatewayDispatchProjectionRequest {
            channel_id: "m9-gateway-unsupported".to_owned(),
            dispatch_request: dispatch_request("m9-gateway-msg-unsupported", 1_709_000_020),
            transport_profile: "grpc".to_owned(),
        },
    );
    assert!(matches!(
        unsupported,
        Err(DataLayerM9GatewayBridgeError::UnsupportedTransport {
            reason_code: DATA_LAYER_M9_GATEWAY_UNSUPPORTED_TRANSPORT_REASON_CODE,
            transport_profile,
        }) if transport_profile == "grpc"
    ));
}

#[test]
fn spec_c04_m9_gateway_presence_projection_fails_closed_for_cross_owner_scope_violation() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let denied = data_layer_m9_gateway_project_presence_event(
        &mut registry,
        DataLayerM9GatewayPresenceProjectionRequest {
            connect_request: presence_connect("kamn:did:agent:alpha-target"),
            query: DataLayerM9PresenceQuery {
                requester_owner_did: "kamn:did:owner:alpha".to_owned(),
                owner_did: "kamn:did:owner:beta".to_owned(),
                requester_agent_did: "kamn:did:agent:alpha-requester".to_owned(),
                target_agent_did: "kamn:did:agent:beta-target".to_owned(),
            },
            transport_profile: "websocket".to_owned(),
        },
    );
    assert!(matches!(
        denied,
        Err(DataLayerM9GatewayBridgeError::RealtimeContract(
            DataLayerM9RealtimeDeliveryError::OwnerScopeViolation {
                reason_code: DATA_LAYER_M9_OWNER_SCOPE_DENIED_REASON_CODE,
            }
        ))
    ));
}

#[test]
fn spec_c05_m9_gateway_presence_projection_fails_closed_for_visibility_denial_without_linkage() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let denied = data_layer_m9_gateway_project_presence_event(
        &mut registry,
        DataLayerM9GatewayPresenceProjectionRequest {
            connect_request: presence_connect("kamn:did:agent:alpha-target"),
            query: DataLayerM9PresenceQuery {
                requester_owner_did: "kamn:did:owner:alpha".to_owned(),
                owner_did: "kamn:did:owner:alpha".to_owned(),
                requester_agent_did: "kamn:did:agent:alpha-requester".to_owned(),
                target_agent_did: "kamn:did:agent:alpha-target".to_owned(),
            },
            transport_profile: "websocket".to_owned(),
        },
    );
    assert!(matches!(
        denied,
        Err(DataLayerM9GatewayBridgeError::RealtimeContract(
            DataLayerM9RealtimeDeliveryError::PresenceVisibilityDenied {
                reason_code: DATA_LAYER_M9_PRESENCE_VISIBILITY_DENIED_REASON_CODE,
            }
        ))
    ));
}

#[test]
fn spec_c06_m9_gateway_dispatch_projection_preserves_anti_spam_fail_closed_reason_codes() {
    let mut channel_store = ChannelStore::new();
    channel_store
        .create_direct(
            "m9-gateway-anti-spam",
            "kamn:did:agent:alpha-sender",
            "kamn:did:agent:alpha-recipient",
        )
        .expect("direct channel should be created");
    let mut anti_spam = AntiSpamEngine::new(AntiSpamConfig::default())
        .expect("default anti-spam config should initialize");
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();

    let denied = data_layer_m9_gateway_project_dispatch_event(
        &mut registry,
        &channel_store,
        &mut anti_spam,
        DataLayerM9GatewayDispatchProjectionRequest {
            channel_id: "m9-gateway-anti-spam".to_owned(),
            dispatch_request: dispatch_request("m9-gateway-anti-spam-msg", 1_709_000_030),
            transport_profile: "websocket".to_owned(),
        },
    );
    assert!(matches!(
        denied,
        Err(DataLayerM9GatewayBridgeError::RealtimeContract(
            DataLayerM9RealtimeDeliveryError::AntiSpamAdmissionDenied {
                reason_code: DATA_LAYER_M9_ANTI_SPAM_INSUFFICIENT_DEPOSIT_REASON_CODE,
            }
        ))
    ));
}
