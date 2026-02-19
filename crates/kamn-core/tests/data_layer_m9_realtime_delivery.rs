use kamn_core::{
    AntiSpamConfig, AntiSpamEngine, ChannelStore, DataLayerM9ChannelDispatchAuthorizationRequest,
    DataLayerM9DispatchAckStatus, DataLayerM9DispatchRequest, DataLayerM9PresenceConnectRequest,
    DataLayerM9PresenceQuery, DataLayerM9PresenceRelationshipRequest,
    DataLayerM9RealtimeDeliveryError, DataLayerM9RealtimeDeliveryRegistry,
    DataLayerM9RuntimeBackpressureProjectionRequest, PeerLifecycleState, RuntimeBackpressureAction,
    DATA_LAYER_M9_ACK_DELIVERED_REASON_CODE, DATA_LAYER_M9_ACK_QUEUED_QUEUE_FULL_REASON_CODE,
    DATA_LAYER_M9_ACK_QUEUED_REASON_CODE, DATA_LAYER_M9_ANTI_SPAM_DUPLICATE_MESSAGE_ID_REASON_CODE,
    DATA_LAYER_M9_ANTI_SPAM_INSUFFICIENT_DEPOSIT_REASON_CODE,
    DATA_LAYER_M9_ANTI_SPAM_RATE_LIMITED_REASON_CODE,
    DATA_LAYER_M9_CHANNEL_MEMBERSHIP_DENIED_REASON_CODE,
    DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES, DATA_LAYER_M9_OWNER_SCOPE_DENIED_REASON_CODE,
    DATA_LAYER_M9_PRESENCE_VISIBILITY_DENIED_REASON_CODE,
    DATA_LAYER_M9_RUNTIME_BACKPRESSURE_INPUT_INVALID_REASON_CODE,
    DATA_LAYER_M9_RUNTIME_BACKPRESSURE_POLICY_INVALID_REASON_CODE,
};

fn dispatch_request(
    requester_owner_did: &str,
    owner_did: &str,
    sender_agent_did: &str,
    recipient_agent_did: &str,
    message_id: &str,
    dispatched_at_epoch_seconds: u64,
) -> DataLayerM9DispatchRequest {
    DataLayerM9DispatchRequest {
        requester_owner_did: requester_owner_did.to_owned(),
        owner_did: owner_did.to_owned(),
        sender_agent_did: sender_agent_did.to_owned(),
        recipient_agent_did: recipient_agent_did.to_owned(),
        message_id: message_id.to_owned(),
        dispatched_at_epoch_seconds,
    }
}

fn enqueue_messages(
    registry: &mut DataLayerM9RealtimeDeliveryRegistry,
    recipient_agent_did: &str,
    count: usize,
    base_epoch_seconds: u64,
) {
    for offset in 0..count {
        let _ = registry
            .dispatch_message(dispatch_request(
                "kamn:did:owner:alpha",
                "kamn:did:owner:alpha",
                "kamn:did:agent:alpha-sender",
                recipient_agent_did,
                format!("m9-bridge-fill-{offset:04}").as_str(),
                base_epoch_seconds + offset as u64,
            ))
            .expect("queue fill dispatch should succeed");
    }
}

#[test]
fn spec_c01_connected_recipient_without_backlog_receives_delivered_ack() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    registry
        .connect_presence(DataLayerM9PresenceConnectRequest {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            agent_did: "kamn:did:agent:alpha-recipient".to_owned(),
            connected_since_epoch_seconds: 1_708_560_100,
            last_heartbeat_epoch_seconds: 1_708_560_100,
            gateway_node: "gateway-a".to_owned(),
            capabilities_active: vec!["ws".to_owned(), "notify".to_owned()],
        })
        .expect("presence connection should succeed");

    let outcome = registry
        .dispatch_message(dispatch_request(
            "kamn:did:owner:alpha",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-sender",
            "kamn:did:agent:alpha-recipient",
            "m9-msg-001",
            1_708_560_110,
        ))
        .expect("dispatch should succeed");

    assert_eq!(outcome.ack_status, DataLayerM9DispatchAckStatus::Delivered);
    assert_eq!(outcome.reason_code, DATA_LAYER_M9_ACK_DELIVERED_REASON_CODE);
    assert_eq!(outcome.pending_queue_depth, 0);
}

#[test]
fn spec_c02_presence_query_is_denied_until_relationship_linkage_is_registered() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    registry
        .connect_presence(DataLayerM9PresenceConnectRequest {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            agent_did: "kamn:did:agent:alpha-target".to_owned(),
            connected_since_epoch_seconds: 1_708_560_100,
            last_heartbeat_epoch_seconds: 1_708_560_100,
            gateway_node: "gateway-a".to_owned(),
            capabilities_active: vec!["ws".to_owned()],
        })
        .expect("presence connection should succeed");

    let denied = registry.query_presence(DataLayerM9PresenceQuery {
        requester_owner_did: "kamn:did:owner:alpha".to_owned(),
        owner_did: "kamn:did:owner:alpha".to_owned(),
        requester_agent_did: "kamn:did:agent:alpha-requester".to_owned(),
        target_agent_did: "kamn:did:agent:alpha-target".to_owned(),
    });
    assert!(matches!(
        denied,
        Err(DataLayerM9RealtimeDeliveryError::PresenceVisibilityDenied {
            reason_code: DATA_LAYER_M9_PRESENCE_VISIBILITY_DENIED_REASON_CODE,
        })
    ));

    registry
        .record_interaction_link(DataLayerM9PresenceRelationshipRequest {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            requester_agent_did: "kamn:did:agent:alpha-requester".to_owned(),
            counterparty_agent_did: "kamn:did:agent:alpha-target".to_owned(),
        })
        .expect("interaction linkage should register");

    let visible = registry
        .query_presence(DataLayerM9PresenceQuery {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            requester_agent_did: "kamn:did:agent:alpha-requester".to_owned(),
            target_agent_did: "kamn:did:agent:alpha-target".to_owned(),
        })
        .expect("presence query should succeed after linkage");

    assert!(visible.is_some());
    let visible_record = visible.expect("presence should be visible");
    assert_eq!(visible_record.agent_did, "kamn:did:agent:alpha-target");
}

#[test]
fn spec_c03_backpressure_thresholds_emit_warning_and_sustained_escalation_markers() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let base = 1_708_560_100;

    for nonce in 0..DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES {
        let message_id = format!("m9-queued-{nonce:04}");
        let outcome = registry
            .dispatch_message(dispatch_request(
                "kamn:did:owner:alpha",
                "kamn:did:owner:alpha",
                "kamn:did:agent:alpha-sender",
                "kamn:did:agent:alpha-recipient",
                message_id.as_str(),
                base,
            ))
            .expect("queue fill dispatch should succeed");
        assert_eq!(outcome.ack_status, DataLayerM9DispatchAckStatus::Queued);
    }

    let warning = registry
        .dispatch_message(dispatch_request(
            "kamn:did:owner:alpha",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-sender",
            "kamn:did:agent:alpha-recipient",
            "m9-warning-threshold",
            base + 301,
        ))
        .expect("warning-threshold dispatch should succeed");
    assert_eq!(
        warning.reason_code,
        DATA_LAYER_M9_ACK_QUEUED_QUEUE_FULL_REASON_CODE
    );
    assert!(warning.backpressure_warning_event);
    assert!(!warning.escrow_timeout_extension_recommended);

    let sustained = registry
        .dispatch_message(dispatch_request(
            "kamn:did:owner:alpha",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-sender",
            "kamn:did:agent:alpha-recipient",
            "m9-sustained-threshold",
            base + 3_601,
        ))
        .expect("sustained-threshold dispatch should succeed");
    assert_eq!(
        sustained.reason_code,
        DATA_LAYER_M9_ACK_QUEUED_QUEUE_FULL_REASON_CODE
    );
    assert!(sustained.backpressure_warning_event);
    assert!(sustained.escrow_timeout_extension_recommended);
}

#[test]
fn spec_c04_cross_owner_dispatch_and_presence_queries_are_denied_fail_closed() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let denied_dispatch = registry.dispatch_message(dispatch_request(
        "kamn:did:owner:intruder",
        "kamn:did:owner:alpha",
        "kamn:did:agent:alpha-sender",
        "kamn:did:agent:alpha-recipient",
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
        requester_owner_did: "kamn:did:owner:intruder".to_owned(),
        owner_did: "kamn:did:owner:alpha".to_owned(),
        requester_agent_did: "kamn:did:agent:alpha-requester".to_owned(),
        target_agent_did: "kamn:did:agent:alpha-target".to_owned(),
    });
    assert!(matches!(
        denied_presence,
        Err(DataLayerM9RealtimeDeliveryError::OwnerScopeViolation {
            reason_code: DATA_LAYER_M9_OWNER_SCOPE_DENIED_REASON_CODE,
        })
    ));
}

#[test]
fn spec_c05_queue_full_dispatch_keeps_pending_cap_and_increments_deferred_counter() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let base = 1_708_560_100;
    for nonce in 0..DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES {
        let _ = registry
            .dispatch_message(dispatch_request(
                "kamn:did:owner:alpha",
                "kamn:did:owner:alpha",
                "kamn:did:agent:alpha-sender",
                "kamn:did:agent:alpha-recipient",
                format!("m9-pending-{nonce:04}").as_str(),
                base,
            ))
            .expect("queue fill dispatch should succeed");
    }

    let first_deferred = registry
        .dispatch_message(dispatch_request(
            "kamn:did:owner:alpha",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-sender",
            "kamn:did:agent:alpha-recipient",
            "m9-deferred-1",
            base + 30,
        ))
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
        .dispatch_message(dispatch_request(
            "kamn:did:owner:alpha",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-sender",
            "kamn:did:agent:alpha-recipient",
            "m9-deferred-2",
            base + 31,
        ))
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

#[test]
fn spec_c06_queue_snapshot_preserves_pending_dispatch_order() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let base = 1_708_560_100;
    for (index, message_id) in ["m9-pending-a", "m9-pending-b", "m9-pending-c"]
        .iter()
        .enumerate()
    {
        let _ = registry
            .dispatch_message(dispatch_request(
                "kamn:did:owner:alpha",
                "kamn:did:owner:alpha",
                "kamn:did:agent:alpha-sender",
                "kamn:did:agent:alpha-recipient",
                message_id,
                base + index as u64,
            ))
            .expect("pending dispatch should succeed");
    }

    let snapshot = registry
        .snapshot_recipient_queue(
            "kamn:did:owner:alpha",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-recipient",
        )
        .expect("queue snapshot should succeed");
    assert_eq!(
        snapshot.pending_message_ids,
        vec![
            "m9-pending-a".to_owned(),
            "m9-pending-b".to_owned(),
            "m9-pending-c".to_owned()
        ]
    );
    assert!(snapshot.deferred_message_ids.is_empty());
}

#[test]
fn spec_c07_queue_snapshot_preserves_deferred_dispatch_order() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let base = 1_708_560_100;
    for nonce in 0..DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES {
        let _ = registry
            .dispatch_message(dispatch_request(
                "kamn:did:owner:alpha",
                "kamn:did:owner:alpha",
                "kamn:did:agent:alpha-sender",
                "kamn:did:agent:alpha-recipient",
                format!("m9-fill-{nonce:04}").as_str(),
                base,
            ))
            .expect("queue fill should succeed");
    }
    for (offset, message_id) in ["m9-deferred-a", "m9-deferred-b", "m9-deferred-c"]
        .iter()
        .enumerate()
    {
        let _ = registry
            .dispatch_message(dispatch_request(
                "kamn:did:owner:alpha",
                "kamn:did:owner:alpha",
                "kamn:did:agent:alpha-sender",
                "kamn:did:agent:alpha-recipient",
                message_id,
                base + 10 + offset as u64,
            ))
            .expect("deferred dispatch should succeed");
    }

    let snapshot = registry
        .snapshot_recipient_queue(
            "kamn:did:owner:alpha",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-recipient",
        )
        .expect("queue snapshot should succeed");
    assert_eq!(
        snapshot.pending_queue_depth,
        DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES
    );
    assert_eq!(
        snapshot.deferred_message_ids,
        vec![
            "m9-deferred-a".to_owned(),
            "m9-deferred-b".to_owned(),
            "m9-deferred-c".to_owned()
        ]
    );
}

#[test]
fn spec_c08_duplicate_message_identifier_is_rejected_fail_closed() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let _ = registry
        .dispatch_message(dispatch_request(
            "kamn:did:owner:alpha",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-sender",
            "kamn:did:agent:alpha-recipient",
            "m9-duplicate-id",
            1_708_560_100,
        ))
        .expect("initial dispatch should succeed");

    let duplicate = registry.dispatch_message(dispatch_request(
        "kamn:did:owner:alpha",
        "kamn:did:owner:alpha",
        "kamn:did:agent:alpha-sender",
        "kamn:did:agent:alpha-recipient",
        "m9-duplicate-id",
        1_708_560_101,
    ));
    assert!(matches!(
        duplicate,
        Err(DataLayerM9RealtimeDeliveryError::DuplicateMessageId(value))
        if value == "m9-duplicate-id"
    ));
}

#[test]
fn spec_c09_channel_dispatch_requires_sender_and_recipient_membership() {
    let mut channel_store = ChannelStore::new();
    channel_store
        .create_direct(
            "m9-direct-1",
            "kamn:did:agent:alpha-sender",
            "kamn:did:agent:alpha-recipient",
        )
        .expect("direct channel should be created");
    let registry = DataLayerM9RealtimeDeliveryRegistry::new();

    registry
        .authorize_channel_dispatch(
            &channel_store,
            DataLayerM9ChannelDispatchAuthorizationRequest {
                requester_owner_did: "kamn:did:owner:alpha".to_owned(),
                owner_did: "kamn:did:owner:alpha".to_owned(),
                channel_id: "m9-direct-1".to_owned(),
                sender_agent_did: "kamn:did:agent:alpha-sender".to_owned(),
                recipient_agent_did: "kamn:did:agent:alpha-recipient".to_owned(),
            },
        )
        .expect("member sender/recipient should authorize");

    let denied = registry.authorize_channel_dispatch(
        &channel_store,
        DataLayerM9ChannelDispatchAuthorizationRequest {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            channel_id: "m9-direct-1".to_owned(),
            sender_agent_did: "kamn:did:agent:alpha-sender".to_owned(),
            recipient_agent_did: "kamn:did:agent:alpha-intruder".to_owned(),
        },
    );
    assert!(matches!(
        denied,
        Err(DataLayerM9RealtimeDeliveryError::ChannelMembershipDenied {
            reason_code: DATA_LAYER_M9_CHANNEL_MEMBERSHIP_DENIED_REASON_CODE,
        })
    ));
}

#[test]
fn spec_c10_dispatch_with_controls_maps_anti_spam_rejections_to_stable_reason_codes() {
    let mut channel_store = ChannelStore::new();
    channel_store
        .create_direct(
            "m9-direct-anti-spam",
            "kamn:did:agent:alpha-sender",
            "kamn:did:agent:alpha-recipient",
        )
        .expect("direct channel should be created");

    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let mut anti_spam = AntiSpamEngine::new(AntiSpamConfig::default())
        .expect("default anti-spam config should initialize");

    let insufficient_deposit = registry.dispatch_message_with_controls(
        &channel_store,
        &mut anti_spam,
        "m9-direct-anti-spam",
        dispatch_request(
            "kamn:did:owner:alpha",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-sender",
            "kamn:did:agent:alpha-recipient",
            "m9-anti-spam-insufficient",
            1_708_560_100,
        ),
    );
    assert!(matches!(
        insufficient_deposit,
        Err(DataLayerM9RealtimeDeliveryError::AntiSpamAdmissionDenied {
            reason_code: DATA_LAYER_M9_ANTI_SPAM_INSUFFICIENT_DEPOSIT_REASON_CODE,
        })
    ));

    anti_spam
        .set_deposit("kamn:did:agent:alpha-sender", 50)
        .expect("sender deposit should be accepted");

    let _ = registry
        .dispatch_message_with_controls(
            &channel_store,
            &mut anti_spam,
            "m9-direct-anti-spam",
            dispatch_request(
                "kamn:did:owner:alpha",
                "kamn:did:owner:alpha",
                "kamn:did:agent:alpha-sender",
                "kamn:did:agent:alpha-recipient",
                "m9-anti-spam-duplicate",
                1_708_560_101,
            ),
        )
        .expect("first dispatch should pass anti-spam");

    let duplicate = registry.dispatch_message_with_controls(
        &channel_store,
        &mut anti_spam,
        "m9-direct-anti-spam",
        dispatch_request(
            "kamn:did:owner:alpha",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-sender",
            "kamn:did:agent:alpha-recipient",
            "m9-anti-spam-duplicate",
            1_708_560_102,
        ),
    );
    assert!(matches!(
        duplicate,
        Err(DataLayerM9RealtimeDeliveryError::AntiSpamAdmissionDenied {
            reason_code: DATA_LAYER_M9_ANTI_SPAM_DUPLICATE_MESSAGE_ID_REASON_CODE,
        })
    ));

    let mut rate_limit_engine = AntiSpamEngine::new(AntiSpamConfig {
        max_messages_per_window: 1,
        window_seconds: 60,
        minimum_sybil_deposit: 1,
        suspension_violation_threshold: 2,
        suspension_seconds: 60,
    })
    .expect("custom anti-spam config should initialize");
    rate_limit_engine
        .set_deposit("kamn:did:agent:alpha-sender", 10)
        .expect("sender deposit should be accepted");
    let _ = registry
        .dispatch_message_with_controls(
            &channel_store,
            &mut rate_limit_engine,
            "m9-direct-anti-spam",
            dispatch_request(
                "kamn:did:owner:alpha",
                "kamn:did:owner:alpha",
                "kamn:did:agent:alpha-sender",
                "kamn:did:agent:alpha-recipient",
                "m9-anti-spam-rate-a",
                1_708_560_200,
            ),
        )
        .expect("first message should pass strict rate policy");

    let rate_limited = registry.dispatch_message_with_controls(
        &channel_store,
        &mut rate_limit_engine,
        "m9-direct-anti-spam",
        dispatch_request(
            "kamn:did:owner:alpha",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-sender",
            "kamn:did:agent:alpha-recipient",
            "m9-anti-spam-rate-b",
            1_708_560_201,
        ),
    );
    assert!(matches!(
        rate_limited,
        Err(DataLayerM9RealtimeDeliveryError::AntiSpamAdmissionDenied {
            reason_code: DATA_LAYER_M9_ANTI_SPAM_RATE_LIMITED_REASON_CODE,
        })
    ));
}

#[test]
fn spec_c11_dispatch_with_controls_allows_member_sender_when_anti_spam_accepts() {
    let mut channel_store = ChannelStore::new();
    channel_store
        .create_direct(
            "m9-direct-allow",
            "kamn:did:agent:alpha-sender",
            "kamn:did:agent:alpha-recipient",
        )
        .expect("direct channel should be created");
    let mut anti_spam = AntiSpamEngine::new(AntiSpamConfig::default())
        .expect("default anti-spam config should initialize");
    anti_spam
        .set_deposit("kamn:did:agent:alpha-sender", 100)
        .expect("sender deposit should be accepted");

    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let outcome = registry
        .dispatch_message_with_controls(
            &channel_store,
            &mut anti_spam,
            "m9-direct-allow",
            dispatch_request(
                "kamn:did:owner:alpha",
                "kamn:did:owner:alpha",
                "kamn:did:agent:alpha-sender",
                "kamn:did:agent:alpha-recipient",
                "m9-controls-allow",
                1_708_560_300,
            ),
        )
        .expect("combined controls dispatch should succeed");

    assert_eq!(outcome.ack_status, DataLayerM9DispatchAckStatus::Queued);
    assert_eq!(outcome.reason_code, DATA_LAYER_M9_ACK_QUEUED_REASON_CODE);
}

#[test]
fn spec_c12_runtime_backpressure_projection_maps_accept_slow_reject_and_purge_actions() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let base = 1_708_560_500;

    let accept = registry
        .project_runtime_backpressure_for_recipient(
            DataLayerM9RuntimeBackpressureProjectionRequest {
                requester_owner_did: "kamn:did:owner:alpha".to_owned(),
                owner_did: "kamn:did:owner:alpha".to_owned(),
                recipient_agent_did: "kamn:did:agent:alpha-accept".to_owned(),
                queue_capacity: 10,
                lifecycle_state: PeerLifecycleState::Active,
                slow_threshold_per_mille: 700,
                reject_threshold_per_mille: 900,
                purge_disconnected_with_pending_queue: true,
            },
        )
        .expect("accept projection should succeed");
    assert_eq!(
        accept.runtime_decision.action,
        RuntimeBackpressureAction::Accept
    );
    assert_eq!(accept.runtime_decision.reason_code(), accept.reason_code);

    enqueue_messages(&mut registry, "kamn:did:agent:alpha-slow", 8, base + 10);
    let slow = registry
        .project_runtime_backpressure_for_recipient(
            DataLayerM9RuntimeBackpressureProjectionRequest {
                requester_owner_did: "kamn:did:owner:alpha".to_owned(),
                owner_did: "kamn:did:owner:alpha".to_owned(),
                recipient_agent_did: "kamn:did:agent:alpha-slow".to_owned(),
                queue_capacity: 10,
                lifecycle_state: PeerLifecycleState::Active,
                slow_threshold_per_mille: 700,
                reject_threshold_per_mille: 900,
                purge_disconnected_with_pending_queue: true,
            },
        )
        .expect("slow projection should succeed");
    assert_eq!(
        slow.runtime_decision.action,
        RuntimeBackpressureAction::SlowProducer
    );

    enqueue_messages(&mut registry, "kamn:did:agent:alpha-reject", 10, base + 30);
    let reject = registry
        .project_runtime_backpressure_for_recipient(
            DataLayerM9RuntimeBackpressureProjectionRequest {
                requester_owner_did: "kamn:did:owner:alpha".to_owned(),
                owner_did: "kamn:did:owner:alpha".to_owned(),
                recipient_agent_did: "kamn:did:agent:alpha-reject".to_owned(),
                queue_capacity: 10,
                lifecycle_state: PeerLifecycleState::Active,
                slow_threshold_per_mille: 700,
                reject_threshold_per_mille: 900,
                purge_disconnected_with_pending_queue: true,
            },
        )
        .expect("reject projection should succeed");
    assert_eq!(
        reject.runtime_decision.action,
        RuntimeBackpressureAction::RejectNewEnqueue
    );

    enqueue_messages(&mut registry, "kamn:did:agent:alpha-purge", 2, base + 50);
    let purge = registry
        .project_runtime_backpressure_for_recipient(
            DataLayerM9RuntimeBackpressureProjectionRequest {
                requester_owner_did: "kamn:did:owner:alpha".to_owned(),
                owner_did: "kamn:did:owner:alpha".to_owned(),
                recipient_agent_did: "kamn:did:agent:alpha-purge".to_owned(),
                queue_capacity: 10,
                lifecycle_state: PeerLifecycleState::Disconnected,
                slow_threshold_per_mille: 700,
                reject_threshold_per_mille: 900,
                purge_disconnected_with_pending_queue: true,
            },
        )
        .expect("purge projection should succeed");
    assert_eq!(
        purge.runtime_decision.action,
        RuntimeBackpressureAction::PurgeStalePeerQueue
    );
}

#[test]
fn spec_c13_runtime_backpressure_projection_fails_closed_for_invalid_policy_and_input() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    enqueue_messages(
        &mut registry,
        "kamn:did:agent:alpha-invalid",
        2,
        1_708_560_700,
    );

    let invalid_policy = registry.project_runtime_backpressure_for_recipient(
        DataLayerM9RuntimeBackpressureProjectionRequest {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            recipient_agent_did: "kamn:did:agent:alpha-invalid".to_owned(),
            queue_capacity: 10,
            lifecycle_state: PeerLifecycleState::Active,
            slow_threshold_per_mille: 900,
            reject_threshold_per_mille: 900,
            purge_disconnected_with_pending_queue: true,
        },
    );
    assert!(matches!(
        invalid_policy,
        Err(
            DataLayerM9RealtimeDeliveryError::RuntimeBackpressurePolicyInvalid {
                reason_code: DATA_LAYER_M9_RUNTIME_BACKPRESSURE_POLICY_INVALID_REASON_CODE,
                ..
            }
        )
    ));

    let invalid_input = registry.project_runtime_backpressure_for_recipient(
        DataLayerM9RuntimeBackpressureProjectionRequest {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            recipient_agent_did: "kamn:did:agent:alpha-invalid".to_owned(),
            queue_capacity: 1,
            lifecycle_state: PeerLifecycleState::Active,
            slow_threshold_per_mille: 700,
            reject_threshold_per_mille: 900,
            purge_disconnected_with_pending_queue: true,
        },
    );
    assert!(matches!(
        invalid_input,
        Err(
            DataLayerM9RealtimeDeliveryError::RuntimeBackpressureInputInvalid {
                reason_code: DATA_LAYER_M9_RUNTIME_BACKPRESSURE_INPUT_INVALID_REASON_CODE,
                ..
            }
        )
    ));
}
