use super::*;

const OWNER_ALPHA: &str = "kamn:did:owner:alpha";
const SENDER_ALPHA: &str = "kamn:did:agent:alpha-sender";
const RECIPIENT_ALPHA: &str = "kamn:did:agent:alpha-recipient";
const CHANNEL_DIRECT_ONE: &str = "m9-direct-1";

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

pub(super) fn run_spec_c06_queue_snapshot_preserves_pending_dispatch_order() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let base = 1_708_560_100;
    for (index, message_id) in ["m9-pending-a", "m9-pending-b", "m9-pending-c"]
        .iter()
        .enumerate()
    {
        let _ = registry
            .dispatch_message(alpha_dispatch_request(message_id, base + index as u64))
            .expect("pending dispatch should succeed");
    }

    let snapshot = registry
        .snapshot_recipient_queue(OWNER_ALPHA, OWNER_ALPHA, RECIPIENT_ALPHA)
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

pub(super) fn run_spec_c07_queue_snapshot_preserves_deferred_dispatch_order() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let base = 1_708_560_100;
    for nonce in 0..DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES {
        let _ = registry
            .dispatch_message(alpha_dispatch_request(
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
            .dispatch_message(alpha_dispatch_request(
                message_id,
                base + 10 + offset as u64,
            ))
            .expect("deferred dispatch should succeed");
    }

    let snapshot = registry
        .snapshot_recipient_queue(OWNER_ALPHA, OWNER_ALPHA, RECIPIENT_ALPHA)
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

pub(super) fn run_spec_c08_duplicate_message_identifier_is_rejected_fail_closed() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let _ = registry
        .dispatch_message(alpha_dispatch_request("m9-duplicate-id", 1_708_560_100))
        .expect("initial dispatch should succeed");

    let duplicate =
        registry.dispatch_message(alpha_dispatch_request("m9-duplicate-id", 1_708_560_101));
    assert!(matches!(
        duplicate,
        Err(DataLayerM9RealtimeDeliveryError::DuplicateMessageId(value))
        if value == "m9-duplicate-id"
    ));
}

pub(super) fn run_spec_c09_channel_dispatch_requires_sender_and_recipient_membership() {
    let mut channel_store = ChannelStore::new();
    channel_store
        .create_direct(CHANNEL_DIRECT_ONE, SENDER_ALPHA, RECIPIENT_ALPHA)
        .expect("direct channel should be created");
    let registry = DataLayerM9RealtimeDeliveryRegistry::new();

    registry
        .authorize_channel_dispatch(
            &channel_store,
            DataLayerM9ChannelDispatchAuthorizationRequest {
                requester_owner_did: OWNER_ALPHA.to_owned(),
                owner_did: OWNER_ALPHA.to_owned(),
                channel_id: CHANNEL_DIRECT_ONE.to_owned(),
                sender_agent_did: SENDER_ALPHA.to_owned(),
                recipient_agent_did: RECIPIENT_ALPHA.to_owned(),
            },
        )
        .expect("member sender/recipient should authorize");

    let denied = registry.authorize_channel_dispatch(
        &channel_store,
        DataLayerM9ChannelDispatchAuthorizationRequest {
            requester_owner_did: OWNER_ALPHA.to_owned(),
            owner_did: OWNER_ALPHA.to_owned(),
            channel_id: CHANNEL_DIRECT_ONE.to_owned(),
            sender_agent_did: SENDER_ALPHA.to_owned(),
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
