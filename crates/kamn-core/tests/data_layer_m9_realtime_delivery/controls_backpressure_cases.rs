use super::*;

const OWNER_ALPHA: &str = "kamn:did:owner:alpha";
const SENDER_ALPHA: &str = "kamn:did:agent:alpha-sender";
const RECIPIENT_ALPHA: &str = "kamn:did:agent:alpha-recipient";
const CHANNEL_ANTI_SPAM: &str = "m9-direct-anti-spam";
const CHANNEL_ALLOW: &str = "m9-direct-allow";

fn alpha_dispatch_request(
    message_id: &str,
    dispatched_at_epoch_seconds: u64,
) -> DataLayerM9DispatchRequest {
    dispatch_request(
        OWNER_ALPHA,
        OWNER_ALPHA,
        SENDER_ALPHA,
        RECIPIENT_ALPHA,
        message_id,
        dispatched_at_epoch_seconds,
    )
}

pub(super) fn run_spec_c10_dispatch_with_controls_maps_anti_spam_rejections_to_stable_reason_codes()
{
    let mut channel_store = ChannelStore::new();
    channel_store
        .create_direct(CHANNEL_ANTI_SPAM, SENDER_ALPHA, RECIPIENT_ALPHA)
        .expect("direct channel should be created");

    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let mut anti_spam = AntiSpamEngine::new(AntiSpamConfig::default())
        .expect("default anti-spam config should initialize");

    let insufficient_deposit = registry.dispatch_message_with_controls(
        &channel_store,
        &mut anti_spam,
        CHANNEL_ANTI_SPAM,
        alpha_dispatch_request("m9-anti-spam-insufficient", 1_708_560_100),
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
            CHANNEL_ANTI_SPAM,
            alpha_dispatch_request("m9-anti-spam-duplicate", 1_708_560_101),
        )
        .expect("first dispatch should pass anti-spam");

    let duplicate = registry.dispatch_message_with_controls(
        &channel_store,
        &mut anti_spam,
        CHANNEL_ANTI_SPAM,
        alpha_dispatch_request("m9-anti-spam-duplicate", 1_708_560_102),
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
        max_seen_message_ids: 10_000,
    })
    .expect("custom anti-spam config should initialize");
    rate_limit_engine
        .set_deposit(SENDER_ALPHA, 10)
        .expect("sender deposit should be accepted");
    let _ = registry
        .dispatch_message_with_controls(
            &channel_store,
            &mut rate_limit_engine,
            CHANNEL_ANTI_SPAM,
            alpha_dispatch_request("m9-anti-spam-rate-a", 1_708_560_200),
        )
        .expect("first message should pass strict rate policy");

    let rate_limited = registry.dispatch_message_with_controls(
        &channel_store,
        &mut rate_limit_engine,
        CHANNEL_ANTI_SPAM,
        alpha_dispatch_request("m9-anti-spam-rate-b", 1_708_560_201),
    );
    assert!(matches!(
        rate_limited,
        Err(DataLayerM9RealtimeDeliveryError::AntiSpamAdmissionDenied {
            reason_code: DATA_LAYER_M9_ANTI_SPAM_RATE_LIMITED_REASON_CODE,
        })
    ));
}

pub(super) fn run_spec_c11_dispatch_with_controls_allows_member_sender_when_anti_spam_accepts() {
    let mut channel_store = ChannelStore::new();
    channel_store
        .create_direct(CHANNEL_ALLOW, SENDER_ALPHA, RECIPIENT_ALPHA)
        .expect("direct channel should be created");
    let mut anti_spam = AntiSpamEngine::new(AntiSpamConfig::default())
        .expect("default anti-spam config should initialize");
    anti_spam
        .set_deposit(SENDER_ALPHA, 100)
        .expect("sender deposit should be accepted");

    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let outcome = registry
        .dispatch_message_with_controls(
            &channel_store,
            &mut anti_spam,
            CHANNEL_ALLOW,
            alpha_dispatch_request("m9-controls-allow", 1_708_560_300),
        )
        .expect("combined controls dispatch should succeed");

    assert_eq!(outcome.ack_status, DataLayerM9DispatchAckStatus::Queued);
    assert_eq!(outcome.reason_code, DATA_LAYER_M9_ACK_QUEUED_REASON_CODE);
}

pub(super) fn run_spec_c12_runtime_backpressure_projection_maps_accept_slow_reject_and_purge_actions(
) {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let base = 1_708_560_500;

    let accept = registry
        .project_runtime_backpressure_for_recipient(
            DataLayerM9RuntimeBackpressureProjectionRequest {
                requester_owner_did: OWNER_ALPHA.to_owned(),
                owner_did: OWNER_ALPHA.to_owned(),
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
                requester_owner_did: OWNER_ALPHA.to_owned(),
                owner_did: OWNER_ALPHA.to_owned(),
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
                requester_owner_did: OWNER_ALPHA.to_owned(),
                owner_did: OWNER_ALPHA.to_owned(),
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
                requester_owner_did: OWNER_ALPHA.to_owned(),
                owner_did: OWNER_ALPHA.to_owned(),
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

pub(super) fn run_spec_c13_runtime_backpressure_projection_fails_closed_for_invalid_policy_and_input(
) {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    enqueue_messages(
        &mut registry,
        "kamn:did:agent:alpha-invalid",
        2,
        1_708_560_700,
    );

    let invalid_policy = registry.project_runtime_backpressure_for_recipient(
        DataLayerM9RuntimeBackpressureProjectionRequest {
            requester_owner_did: OWNER_ALPHA.to_owned(),
            owner_did: OWNER_ALPHA.to_owned(),
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
            requester_owner_did: OWNER_ALPHA.to_owned(),
            owner_did: OWNER_ALPHA.to_owned(),
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
