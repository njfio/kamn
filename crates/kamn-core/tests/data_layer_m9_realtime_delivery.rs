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
    DATA_LAYER_M9_INVALID_RECIPIENT_AGENT_DID_REASON_CODE,
    DATA_LAYER_M9_INVALID_REQUESTER_AGENT_DID_REASON_CODE,
    DATA_LAYER_M9_INVALID_REQUESTER_OWNER_DID_REASON_CODE,
    DATA_LAYER_M9_INVALID_SENDER_AGENT_DID_REASON_CODE,
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

#[path = "data_layer_m9_realtime_delivery/baseline_flow_cases.rs"]
mod baseline_flow_cases;
#[path = "data_layer_m9_realtime_delivery/controls_backpressure_cases.rs"]
mod controls_backpressure_cases;
#[path = "data_layer_m9_realtime_delivery/input_validation_cases.rs"]
mod input_validation_cases;
#[path = "data_layer_m9_realtime_delivery/queue_channel_cases.rs"]
mod queue_channel_cases;

#[test]
fn spec_c01_connected_recipient_without_backlog_receives_delivered_ack() {
    baseline_flow_cases::run_spec_c01_connected_recipient_without_backlog_receives_delivered_ack();
}

#[test]
fn spec_c02_presence_query_is_denied_until_relationship_linkage_is_registered() {
    baseline_flow_cases::run_spec_c02_presence_query_is_denied_until_relationship_linkage_is_registered();
}

#[test]
fn spec_c03_backpressure_thresholds_emit_warning_and_sustained_escalation_markers() {
    baseline_flow_cases::run_spec_c03_backpressure_thresholds_emit_warning_and_sustained_escalation_markers();
}

#[test]
fn spec_c04_cross_owner_dispatch_and_presence_queries_are_denied_fail_closed() {
    baseline_flow_cases::run_spec_c04_cross_owner_dispatch_and_presence_queries_are_denied_fail_closed();
}

#[test]
fn spec_c05_queue_full_dispatch_keeps_pending_cap_and_increments_deferred_counter() {
    baseline_flow_cases::run_spec_c05_queue_full_dispatch_keeps_pending_cap_and_increments_deferred_counter();
}

#[test]
fn spec_c06_queue_snapshot_preserves_pending_dispatch_order() {
    queue_channel_cases::run_spec_c06_queue_snapshot_preserves_pending_dispatch_order();
}

#[test]
fn spec_c07_queue_snapshot_preserves_deferred_dispatch_order() {
    queue_channel_cases::run_spec_c07_queue_snapshot_preserves_deferred_dispatch_order();
}

#[test]
fn spec_c08_duplicate_message_identifier_is_rejected_fail_closed() {
    queue_channel_cases::run_spec_c08_duplicate_message_identifier_is_rejected_fail_closed();
}

#[test]
fn spec_c09_channel_dispatch_requires_sender_and_recipient_membership() {
    queue_channel_cases::run_spec_c09_channel_dispatch_requires_sender_and_recipient_membership();
}

#[test]
fn spec_c10_dispatch_with_controls_maps_anti_spam_rejections_to_stable_reason_codes() {
    controls_backpressure_cases::run_spec_c10_dispatch_with_controls_maps_anti_spam_rejections_to_stable_reason_codes();
}

#[test]
fn spec_c11_dispatch_with_controls_allows_member_sender_when_anti_spam_accepts() {
    controls_backpressure_cases::run_spec_c11_dispatch_with_controls_allows_member_sender_when_anti_spam_accepts();
}

#[test]
fn spec_c12_runtime_backpressure_projection_maps_accept_slow_reject_and_purge_actions() {
    controls_backpressure_cases::run_spec_c12_runtime_backpressure_projection_maps_accept_slow_reject_and_purge_actions();
}

#[test]
fn spec_c13_runtime_backpressure_projection_fails_closed_for_invalid_policy_and_input() {
    controls_backpressure_cases::run_spec_c13_runtime_backpressure_projection_fails_closed_for_invalid_policy_and_input();
}

#[test]
fn spec_c14_invalid_requester_owner_did_fails_closed_with_field_taxonomy() {
    input_validation_cases::run_spec_c14_invalid_requester_owner_did_fails_closed_with_field_taxonomy();
}

#[test]
fn spec_c15_invalid_sender_and_recipient_agent_dids_fail_closed_with_field_taxonomy() {
    input_validation_cases::run_spec_c15_invalid_sender_and_recipient_agent_dids_fail_closed_with_field_taxonomy();
}

#[test]
fn spec_c16_invalid_presence_requester_agent_did_fails_closed_with_field_taxonomy() {
    input_validation_cases::run_spec_c16_invalid_presence_requester_agent_did_fails_closed_with_field_taxonomy();
}
