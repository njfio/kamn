use kamn_core::{
    ChannelPermissionEngine, ChannelPermissions, DeliveryGuardInput, DeliveryValidationResult,
    DurableGuardBundleSnapshotStore, DurableGuardSnapshotBundle, DurableGuardSnapshotStoreError,
    FileDurableGuardSnapshotStore, InMemoryDurableGuardSnapshotStore, MessageDeliveryGuards,
    PermissionRule, RetentionMessage, RetentionPolicy,
};
use std::fs;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

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

fn tmp_file_path(prefix: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "kamn-{prefix}-{}-{now}.snapshot",
        std::process::id()
    ))
}

#[test]
fn unit_bundle_schema_mismatch_is_rejected() {
    let guards = MessageDeliveryGuards::new();
    let channels = ChannelPermissionEngine::new();
    let mut bundle = DurableGuardSnapshotBundle::capture(&guards, &channels);
    bundle.schema_version = bundle.schema_version.saturating_add(1);

    let mut restored_guards = MessageDeliveryGuards::new();
    let mut restored_channels = ChannelPermissionEngine::new();
    assert_eq!(
        bundle.restore_into(&mut restored_guards, &mut restored_channels),
        Err(
            DurableGuardSnapshotStoreError::BundleSchemaVersionMismatch {
                expected: 1,
                found: 2,
            }
        )
    );
}

#[test]
fn functional_in_memory_bundle_store_roundtrip() {
    let mut guards = MessageDeliveryGuards::new();
    let mut channels = ChannelPermissionEngine::new();
    register_channel(
        &mut channels,
        "channel:group:store-roundtrip",
        RetentionPolicy::MaxMessageCount(4),
    );
    assert_eq!(
        guards.validate(delivery_input(
            "urn:uuid:snapshot-store-msg-1",
            1,
            "2026-02-09T00:05:00.000Z",
        )),
        DeliveryValidationResult::Accepted
    );

    let bundle = DurableGuardSnapshotBundle::capture(&guards, &channels);
    let mut store = InMemoryDurableGuardSnapshotStore::default();
    store
        .save_bundle(bundle.clone())
        .expect("bundle save should pass");

    assert_eq!(
        store.load_bundle().expect("bundle load should pass"),
        Some(bundle)
    );
}

#[test]
fn integration_file_bundle_restore_preserves_invariants() {
    let mut guards = MessageDeliveryGuards::new();
    let mut channels = ChannelPermissionEngine::new();
    register_channel(
        &mut channels,
        "channel:group:file-restore",
        RetentionPolicy::MaxAgeSeconds(300),
    );
    assert_eq!(
        guards.validate(delivery_input(
            "urn:uuid:file-restore-msg-1",
            1,
            "2026-02-09T00:10:00.000Z",
        )),
        DeliveryValidationResult::Accepted
    );

    let path = tmp_file_path("durable-guard-bundle");
    let mut store = FileDurableGuardSnapshotStore::new(path.clone()).expect("store should build");
    store
        .save_bundle(DurableGuardSnapshotBundle::capture(&guards, &channels))
        .expect("bundle save should pass");

    let bundle = store
        .load_bundle()
        .expect("bundle load should pass")
        .expect("bundle should exist");
    let mut restored_guards = MessageDeliveryGuards::new();
    let mut restored_channels = ChannelPermissionEngine::new();
    bundle
        .restore_into(&mut restored_guards, &mut restored_channels)
        .expect("bundle restore should pass");

    assert_eq!(restored_guards.expected_nonce("kamn:did:agent:sender-1"), 2);
    let candidates = restored_channels
        .retention_candidates(
            "channel:group:file-restore",
            1_000,
            vec![
                RetentionMessage {
                    id: "msg-a".to_owned(),
                    created_at_secs: 100,
                },
                RetentionMessage {
                    id: "msg-b".to_owned(),
                    created_at_secs: 800,
                },
            ],
        )
        .expect("retention should evaluate");
    assert_eq!(candidates, vec!["msg-a".to_owned()]);

    let _ = fs::remove_file(path);
}

#[test]
fn regression_truncated_bundle_payload_rejected() {
    // Regression: #679
    let path = tmp_file_path("durable-guard-corrupt");
    fs::write(&path, "bundle_schema|1\nchannel_begin|broken\n").expect("fixture write should pass");

    let store = FileDurableGuardSnapshotStore::new(path.clone()).expect("store should build");
    match store.load_bundle() {
        Err(DurableGuardSnapshotStoreError::InvalidPayload(_)) => {}
        other => panic!("expected invalid payload error, got {other:?}"),
    }

    let _ = fs::remove_file(path);
}

#[test]
fn performance_bundle_contract_lane_budget() {
    let mut guards = MessageDeliveryGuards::new();
    let mut channels = ChannelPermissionEngine::new();
    register_channel(
        &mut channels,
        "channel:group:bundle-perf",
        RetentionPolicy::MaxMessageCount(64),
    );
    for nonce in 1..=256 {
        assert_eq!(
            guards.validate(delivery_input(
                &format!("urn:uuid:bundle-perf-{nonce}"),
                nonce,
                "2026-02-09T00:10:00.000Z",
            )),
            DeliveryValidationResult::Accepted
        );
    }

    let path = tmp_file_path("durable-guard-perf");
    let mut store = FileDurableGuardSnapshotStore::new(path.clone()).expect("store should build");
    let start = Instant::now();
    let bundle = DurableGuardSnapshotBundle::capture(&guards, &channels);
    store.save_bundle(bundle).expect("bundle save should pass");
    let _ = store.load_bundle().expect("bundle load should pass");
    let elapsed_ms = start.elapsed().as_millis();

    assert!(
        elapsed_ms < 500,
        "durable guard bundle store contract lane exceeded budget: {elapsed_ms}ms"
    );
    let _ = fs::remove_file(path);
}

#[test]
#[ignore = "scheduled durable guard store deep lane"]
fn performance_bundle_store_deep_lane_stress() {
    let mut guards = MessageDeliveryGuards::new();
    let mut channels = ChannelPermissionEngine::new();
    register_channel(
        &mut channels,
        "channel:group:bundle-deep",
        RetentionPolicy::MaxMessageCount(1024),
    );
    for nonce in 1..=5_000 {
        assert_eq!(
            guards.validate(delivery_input(
                &format!("urn:uuid:bundle-deep-{nonce}"),
                nonce,
                "2026-02-09T00:20:00.000Z",
            )),
            DeliveryValidationResult::Accepted
        );
    }

    let path = tmp_file_path("durable-guard-deep");
    let mut store = FileDurableGuardSnapshotStore::new(path.clone()).expect("store should build");
    let bundle = DurableGuardSnapshotBundle::capture(&guards, &channels);
    store.save_bundle(bundle).expect("bundle save should pass");
    let loaded = store
        .load_bundle()
        .expect("bundle load should pass")
        .expect("bundle should exist");
    let mut restored_guards = MessageDeliveryGuards::new();
    let mut restored_channels = ChannelPermissionEngine::new();
    loaded
        .restore_into(&mut restored_guards, &mut restored_channels)
        .expect("deep restore should pass");
    assert_eq!(
        restored_guards.expected_nonce("kamn:did:agent:sender-1"),
        5_001
    );
    let _ = fs::remove_file(path);
}
