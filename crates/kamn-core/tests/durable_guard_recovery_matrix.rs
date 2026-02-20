use kamn_core::{
    ChannelPermissionEngine, ChannelPermissions, ChannelPolicyError, ChannelPolicySnapshotError,
    DeliveryFailureCode, DeliveryGuardInput, DeliveryGuardSnapshotError, DeliveryValidationResult,
    MessageDeliveryGuards, PermissionRule, RetentionMessage, RetentionPolicy,
};
use std::time::Instant;

fn delivery_input(message_id: &str, nonce: u64, received_at: &str) -> DeliveryGuardInput {
    DeliveryGuardInput {
        message_id: message_id.to_owned(),
        sender: "kamn:did:agent:sender-1".to_owned(),
        recipient: "kamn:did:agent:recipient-1".to_owned(),
        nonce,
        created: "2026-02-09T00:00:00.000Z".to_owned(),
        expires: "2026-02-09T00:30:00.000Z".to_owned(),
        received_at: received_at.to_owned(),
    }
}

fn channel_permissions(retention: RetentionPolicy) -> ChannelPermissions {
    ChannelPermissions {
        send: PermissionRule::Members,
        read: PermissionRule::Members,
        invite: PermissionRule::Admins,
        remove: PermissionRule::Admins,
        configure: PermissionRule::Admins,
        retention,
    }
}

fn register_channel(
    engine: &mut ChannelPermissionEngine,
    channel_id: &str,
    retention: RetentionPolicy,
) {
    engine
        .register_channel(
            channel_id,
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:member-1".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
            channel_permissions(retention),
        )
        .expect("channel should register");
}

#[test]
fn unit_delivery_guard_snapshot_rejects_schema_mismatch() {
    let guards = MessageDeliveryGuards::new();
    let mut snapshot = guards.export_snapshot();
    snapshot.schema_version = snapshot.schema_version.saturating_add(1);

    let mut restored = MessageDeliveryGuards::new();
    assert_eq!(
        restored.restore_snapshot(snapshot),
        Err(DeliveryGuardSnapshotError::SchemaVersionMismatch {
            expected: 1,
            found: 2,
        })
    );
}

#[test]
fn unit_channel_policy_snapshot_rejects_schema_mismatch() {
    let engine = ChannelPermissionEngine::new();
    let mut snapshot = engine.export_snapshot();
    snapshot.schema_version = snapshot.schema_version.saturating_add(1);

    let mut restored = ChannelPermissionEngine::new();
    assert_eq!(
        restored.restore_snapshot(snapshot),
        Err(ChannelPolicySnapshotError::SchemaVersionMismatch {
            expected: 1,
            found: 2,
        })
    );
}

#[test]
fn functional_delivery_guard_recovery_restores_nonce_and_replay_state() {
    let mut guards = MessageDeliveryGuards::new();
    assert_eq!(
        guards.validate(delivery_input(
            "urn:uuid:durable-msg-1",
            1,
            "2026-02-09T00:10:00.000Z"
        )),
        DeliveryValidationResult::Accepted
    );
    assert_eq!(
        guards.validate(delivery_input(
            "urn:uuid:durable-msg-2",
            2,
            "2026-02-09T00:11:00.000Z"
        )),
        DeliveryValidationResult::Accepted
    );

    let snapshot = guards.export_snapshot();
    let mut restored = MessageDeliveryGuards::new();
    restored
        .restore_snapshot(snapshot)
        .expect("delivery snapshot restore should succeed");

    assert_eq!(restored.expected_nonce("kamn:did:agent:sender-1"), 3);
    match restored.validate(delivery_input(
        "urn:uuid:durable-msg-1",
        3,
        "2026-02-09T00:12:00.000Z",
    )) {
        DeliveryValidationResult::Rejected(notice) => {
            assert_eq!(notice.code, DeliveryFailureCode::Replay);
        }
        DeliveryValidationResult::Accepted => panic!("expected replay rejection after restore"),
    }
}

#[test]
fn functional_channel_policy_recovery_restores_retention_candidates() {
    let mut engine = ChannelPermissionEngine::new();
    register_channel(
        &mut engine,
        "channel:group:durable-retention",
        RetentionPolicy::MaxMessageCount(2),
    );

    let snapshot = engine.export_snapshot();
    let mut restored = ChannelPermissionEngine::new();
    restored
        .restore_snapshot(snapshot)
        .expect("channel policy snapshot restore should succeed");

    let candidates = restored
        .retention_candidates(
            "channel:group:durable-retention",
            1_000,
            vec![
                RetentionMessage {
                    id: "msg-a".to_owned(),
                    created_at_secs: 100,
                },
                RetentionMessage {
                    id: "msg-b".to_owned(),
                    created_at_secs: 200,
                },
                RetentionMessage {
                    id: "msg-c".to_owned(),
                    created_at_secs: 300,
                },
            ],
        )
        .expect("retention should evaluate after restore");
    assert_eq!(candidates, vec!["msg-a".to_owned()]);
}

#[test]
fn integration_durable_guard_recovery_matrix_restores_delivery_and_retention_invariants() {
    let mut guards = MessageDeliveryGuards::new();
    let mut channel_engine = ChannelPermissionEngine::new();
    register_channel(
        &mut channel_engine,
        "channel:group:durable-integration",
        RetentionPolicy::MaxAgeSeconds(300),
    );

    assert_eq!(
        guards.validate(delivery_input(
            "urn:uuid:durable-integration-msg-1",
            1,
            "2026-02-09T00:05:00.000Z",
        )),
        DeliveryValidationResult::Accepted
    );

    let guard_snapshot = guards.export_snapshot();
    let channel_snapshot = channel_engine.export_snapshot();

    let mut restored_guards = MessageDeliveryGuards::new();
    restored_guards
        .restore_snapshot(guard_snapshot)
        .expect("guard snapshot should restore");
    let mut restored_channel_engine = ChannelPermissionEngine::new();
    restored_channel_engine
        .restore_snapshot(channel_snapshot)
        .expect("channel snapshot should restore");

    assert_eq!(restored_guards.expected_nonce("kamn:did:agent:sender-1"), 2);
    assert!(restored_channel_engine
        .authorize(
            "channel:group:durable-integration",
            "kamn:did:agent:owner",
            kamn_core::ChannelAction::Invite,
        )
        .is_ok());
}

#[test]
fn regression_corrupted_delivery_snapshot_rejected_with_explicit_error() {
    // Regression: #679
    let guards = MessageDeliveryGuards::new();
    let mut snapshot = guards.export_snapshot();
    snapshot
        .next_nonce_by_sender
        .insert("kamn:did:agent:sender-1".to_owned(), 0);

    let mut restored = MessageDeliveryGuards::new();
    assert_eq!(
        restored.restore_snapshot(snapshot),
        Err(DeliveryGuardSnapshotError::InvalidNonce {
            sender: "kamn:did:agent:sender-1".to_owned(),
            nonce: 0,
        })
    );
}

#[test]
fn regression_corrupted_channel_snapshot_rejected_with_explicit_error() {
    // Regression: #679
    let mut engine = ChannelPermissionEngine::new();
    register_channel(
        &mut engine,
        "channel:group:durable-corrupt",
        RetentionPolicy::Forever,
    );
    let mut snapshot = engine.export_snapshot();
    snapshot.channels[0]
        .admins
        .push("kamn:did:agent:not-member".to_owned());

    let mut restored = ChannelPermissionEngine::new();
    assert_eq!(
        restored.restore_snapshot(snapshot),
        Err(ChannelPolicySnapshotError::ChannelPolicy(
            ChannelPolicyError::AdminNotMember("kamn:did:agent:not-member".to_owned()),
        ))
    );
}

#[test]
fn performance_durable_guard_recovery_contract_lane_budget() {
    let mut guards = MessageDeliveryGuards::new();
    let mut channel_engine = ChannelPermissionEngine::new();
    register_channel(
        &mut channel_engine,
        "channel:group:durable-perf",
        RetentionPolicy::MaxMessageCount(32),
    );

    let start = Instant::now();
    for nonce in 1..=256 {
        assert_eq!(
            guards.validate(delivery_input(
                &format!("urn:uuid:durable-perf-{nonce}"),
                nonce,
                "2026-02-09T00:10:00.000Z",
            )),
            DeliveryValidationResult::Accepted
        );
    }
    let guard_snapshot = guards.export_snapshot();
    let channel_snapshot = channel_engine.export_snapshot();

    let mut restored_guards = MessageDeliveryGuards::new();
    restored_guards
        .restore_snapshot(guard_snapshot)
        .expect("guard restore should pass");
    let mut restored_channel_engine = ChannelPermissionEngine::new();
    restored_channel_engine
        .restore_snapshot(channel_snapshot)
        .expect("channel restore should pass");

    let elapsed_ms = start.elapsed().as_millis();
    assert!(
        elapsed_ms < 500,
        "durable guard recovery contract lane exceeded budget: {elapsed_ms}ms"
    );
}

#[test]
#[ignore = "scheduled durable guard recovery deep matrix"]
fn performance_durable_guard_recovery_matrix_deep_lane() {
    let mut guards = MessageDeliveryGuards::new();
    let mut channel_engine = ChannelPermissionEngine::new();
    register_channel(
        &mut channel_engine,
        "channel:group:durable-deep",
        RetentionPolicy::MaxMessageCount(512),
    );

    for nonce in 1..=5_000 {
        assert_eq!(
            guards.validate(delivery_input(
                &format!("urn:uuid:durable-deep-{nonce}"),
                nonce,
                "2026-02-09T00:15:00.000Z",
            )),
            DeliveryValidationResult::Accepted
        );
    }

    let guard_snapshot = guards.export_snapshot();
    let channel_snapshot = channel_engine.export_snapshot();
    let mut restored_guards = MessageDeliveryGuards::new();
    restored_guards
        .restore_snapshot(guard_snapshot)
        .expect("guard deep restore should pass");
    let mut restored_channel_engine = ChannelPermissionEngine::new();
    restored_channel_engine
        .restore_snapshot(channel_snapshot)
        .expect("channel deep restore should pass");
    assert_eq!(
        restored_guards.expected_nonce("kamn:did:agent:sender-1"),
        5_001
    );
}
