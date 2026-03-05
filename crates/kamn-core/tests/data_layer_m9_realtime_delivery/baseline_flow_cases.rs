use super::*;

const OWNER_ALPHA: &str = "kamn:did:owner:alpha";
const OWNER_INTRUDER: &str = "kamn:did:owner:intruder";
const SENDER_ALPHA: &str = "kamn:did:agent:alpha-sender";
const RECIPIENT_ALPHA: &str = "kamn:did:agent:alpha-recipient";
const REQUESTER_ALPHA: &str = "kamn:did:agent:alpha-requester";
const TARGET_ALPHA: &str = "kamn:did:agent:alpha-target";

// Keeps repeated dispatch setup deterministic across baseline scenarios.
fn alpha_dispatch_request(message_id: &str, dispatched_at_epoch_seconds: u64) -> DataLayerM9DispatchRequest {
    dispatch_request(
        OWNER_ALPHA,
        OWNER_ALPHA,
        SENDER_ALPHA,
        RECIPIENT_ALPHA,
        message_id,
        dispatched_at_epoch_seconds,
    )
}

pub(super) fn run_spec_c01_connected_recipient_without_backlog_receives_delivered_ack() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    registry
        .connect_presence(DataLayerM9PresenceConnectRequest {
            requester_owner_did: OWNER_ALPHA.to_owned(),
            owner_did: OWNER_ALPHA.to_owned(),
            agent_did: RECIPIENT_ALPHA.to_owned(),
            connected_since_epoch_seconds: 1_708_560_100,
            last_heartbeat_epoch_seconds: 1_708_560_100,
            gateway_node: "gateway-a".to_owned(),
            capabilities_active: vec!["ws".to_owned(), "notify".to_owned()],
        })
        .expect("presence connection should succeed");

    let outcome = registry
        .dispatch_message(alpha_dispatch_request("m9-msg-001", 1_708_560_110))
        .expect("dispatch should succeed");

    assert_eq!(outcome.ack_status, DataLayerM9DispatchAckStatus::Delivered);
    assert_eq!(outcome.reason_code, DATA_LAYER_M9_ACK_DELIVERED_REASON_CODE);
    assert_eq!(outcome.pending_queue_depth, 0);
}

pub(super) fn run_spec_c02_presence_query_is_denied_until_relationship_linkage_is_registered() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    registry
        .connect_presence(DataLayerM9PresenceConnectRequest {
            requester_owner_did: OWNER_ALPHA.to_owned(),
            owner_did: OWNER_ALPHA.to_owned(),
            agent_did: TARGET_ALPHA.to_owned(),
            connected_since_epoch_seconds: 1_708_560_100,
            last_heartbeat_epoch_seconds: 1_708_560_100,
            gateway_node: "gateway-a".to_owned(),
            capabilities_active: vec!["ws".to_owned()],
        })
        .expect("presence connection should succeed");

    let denied = registry.query_presence(DataLayerM9PresenceQuery {
        requester_owner_did: OWNER_ALPHA.to_owned(),
        owner_did: OWNER_ALPHA.to_owned(),
        requester_agent_did: REQUESTER_ALPHA.to_owned(),
        target_agent_did: TARGET_ALPHA.to_owned(),
    });
    assert!(matches!(
        denied,
        Err(DataLayerM9RealtimeDeliveryError::PresenceVisibilityDenied {
            reason_code: DATA_LAYER_M9_PRESENCE_VISIBILITY_DENIED_REASON_CODE,
        })
    ));

    registry
        .record_interaction_link(DataLayerM9PresenceRelationshipRequest {
            requester_owner_did: OWNER_ALPHA.to_owned(),
            owner_did: OWNER_ALPHA.to_owned(),
            requester_agent_did: REQUESTER_ALPHA.to_owned(),
            counterparty_agent_did: TARGET_ALPHA.to_owned(),
        })
        .expect("interaction linkage should register");

    let visible = registry
        .query_presence(DataLayerM9PresenceQuery {
            requester_owner_did: OWNER_ALPHA.to_owned(),
            owner_did: OWNER_ALPHA.to_owned(),
            requester_agent_did: REQUESTER_ALPHA.to_owned(),
            target_agent_did: TARGET_ALPHA.to_owned(),
        })
        .expect("presence query should succeed after linkage");

    assert!(visible.is_some());
    let visible_record = visible.expect("presence should be visible");
    assert_eq!(visible_record.agent_did, "kamn:did:agent:alpha-target");
}

pub(super) fn run_spec_c03_backpressure_thresholds_emit_warning_and_sustained_escalation_markers()
{
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let base = 1_708_560_100;

    for nonce in 0..DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES {
        let message_id = format!("m9-queued-{nonce:04}");
        let outcome = registry
            .dispatch_message(alpha_dispatch_request(message_id.as_str(), base))
            .expect("queue fill dispatch should succeed");
        assert_eq!(outcome.ack_status, DataLayerM9DispatchAckStatus::Queued);
    }

    let warning = registry
        .dispatch_message(alpha_dispatch_request("m9-warning-threshold", base + 301))
        .expect("warning-threshold dispatch should succeed");
    assert_eq!(
        warning.reason_code,
        DATA_LAYER_M9_ACK_QUEUED_QUEUE_FULL_REASON_CODE
    );
    assert!(warning.backpressure_warning_event);
    assert!(!warning.escrow_timeout_extension_recommended);

    let sustained = registry
        .dispatch_message(alpha_dispatch_request("m9-sustained-threshold", base + 3_601))
        .expect("sustained-threshold dispatch should succeed");
    assert_eq!(
        sustained.reason_code,
        DATA_LAYER_M9_ACK_QUEUED_QUEUE_FULL_REASON_CODE
    );
    assert!(sustained.backpressure_warning_event);
    assert!(sustained.escrow_timeout_extension_recommended);
}

pub(super) fn run_spec_c04_cross_owner_dispatch_and_presence_queries_are_denied_fail_closed() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let denied_dispatch = registry.dispatch_message(dispatch_request(
        OWNER_INTRUDER,
        OWNER_ALPHA,
        SENDER_ALPHA,
        RECIPIENT_ALPHA,
        "m9-cross-owner",
        1_708_560_100,
    ));
    assert!(matches!(
        denied_dispatch,
        Err(DataLayerM9RealtimeDeliveryError::OwnerScopeViolation {
            reason_code: DATA_LAYER_M9_OWNER_SCOPE_DENIED_REASON_CODE,
        })
    ));

    let denied_presence = registry.query_presence(DataLayerM9PresenceQuery {
        requester_owner_did: OWNER_INTRUDER.to_owned(),
        owner_did: OWNER_ALPHA.to_owned(),
        requester_agent_did: REQUESTER_ALPHA.to_owned(),
        target_agent_did: TARGET_ALPHA.to_owned(),
    });
    assert!(matches!(
        denied_presence,
        Err(DataLayerM9RealtimeDeliveryError::OwnerScopeViolation {
            reason_code: DATA_LAYER_M9_OWNER_SCOPE_DENIED_REASON_CODE,
        })
    ));
}

pub(super) fn run_spec_c05_queue_full_dispatch_keeps_pending_cap_and_increments_deferred_counter() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let base = 1_708_560_100;
    for nonce in 0..DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES {
        let _ = registry
            .dispatch_message(alpha_dispatch_request(format!("m9-pending-{nonce:04}").as_str(), base))
            .expect("queue fill dispatch should succeed");
    }

    let first_deferred = registry
        .dispatch_message(alpha_dispatch_request("m9-deferred-1", base + 30))
        .expect("first deferred dispatch should succeed");
    assert_eq!(
        first_deferred.ack_status,
        DataLayerM9DispatchAckStatus::Queued
    );
    assert_eq!(
        first_deferred.reason_code,
        DATA_LAYER_M9_ACK_QUEUED_QUEUE_FULL_REASON_CODE
    );
    assert_eq!(
        first_deferred.pending_queue_depth,
        DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES
    );
    assert_eq!(first_deferred.deferred_count, 1);

    let second_deferred = registry
        .dispatch_message(alpha_dispatch_request("m9-deferred-2", base + 31))
        .expect("second deferred dispatch should succeed");
    assert_eq!(
        second_deferred.ack_status,
        DataLayerM9DispatchAckStatus::Queued
    );
    assert_eq!(
        second_deferred.reason_code,
        DATA_LAYER_M9_ACK_QUEUED_QUEUE_FULL_REASON_CODE
    );
    assert_eq!(second_deferred.deferred_count, 2);
    assert!(!second_deferred.backpressure_warning_event);
    assert_eq!(
        second_deferred.pending_queue_depth,
        DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES
    );

    assert_ne!(
        DATA_LAYER_M9_ACK_QUEUED_REASON_CODE,
        DATA_LAYER_M9_ACK_QUEUED_QUEUE_FULL_REASON_CODE
    );
}
