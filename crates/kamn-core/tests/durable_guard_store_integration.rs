use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use kamn_core::{
    ChannelPermissionEngine, ChannelPermissions, DeliveryFailureCode, DeliveryGuardInput,
    DeliveryValidationResult, DurableGuardBundleSnapshotStore, DurableGuardSnapshotBundle,
    DurableGuardSnapshotStoreError, FileDurableGuardSnapshotStore,
    InMemoryDurableGuardSnapshotStore, MessageDeliveryGuards, PermissionRule, RetentionPolicy,
    DURABLE_GUARD_BUNDLE_SCHEMA_VERSION,
};

fn valid_permissions() -> ChannelPermissions {
    ChannelPermissions {
        send: PermissionRule::Members,
        read: PermissionRule::Members,
        invite: PermissionRule::Admins,
        remove: PermissionRule::Admins,
        configure: PermissionRule::Admins,
        retention: RetentionPolicy::Forever,
    }
}

fn valid_input(message_id: &str, nonce: u64) -> DeliveryGuardInput {
    DeliveryGuardInput {
        message_id: message_id.to_owned(),
        sender: "kamn:did:agent:sender-1".to_owned(),
        recipient: "kamn:did:agent:recipient-1".to_owned(),
        nonce,
        created: "2026-02-07T20:15:30.123Z".to_owned(),
        expires: "2026-02-07T20:45:30.123Z".to_owned(),
        received_at: "2026-02-07T20:20:30.123Z".to_owned(),
    }
}

fn seeded_bundle() -> DurableGuardSnapshotBundle {
    let mut delivery = MessageDeliveryGuards::new();
    assert_eq!(
        delivery.validate(valid_input("urn:uuid:msg-1", 1)),
        DeliveryValidationResult::Accepted
    );

    let mut channel = ChannelPermissionEngine::new();
    channel
        .register_channel(
            "channel-1",
            vec![
                "kamn:did:agent:sender-1".to_owned(),
                "kamn:did:agent:recipient-1".to_owned(),
            ],
            vec!["kamn:did:agent:sender-1".to_owned()],
            valid_permissions(),
        )
        .expect("channel registration should succeed");

    DurableGuardSnapshotBundle::capture(&delivery, &channel)
}

fn temp_snapshot_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be valid")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-durable-guard-{nanos}.snapshot"))
}

fn remove_if_present(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

#[test]
fn integration_durable_guard_bundle_capture_and_restore_reproduces_guard_state() {
    let bundle = seeded_bundle();
    let original_delivery = bundle.delivery_guard.clone();
    let original_channel = bundle.channel_policy.clone();

    let mut delivery = MessageDeliveryGuards::new();
    let mut channel = ChannelPermissionEngine::new();
    bundle
        .restore_into(&mut delivery, &mut channel)
        .expect("restore should succeed");

    assert_eq!(delivery.export_snapshot(), original_delivery);
    assert_eq!(channel.export_snapshot(), original_channel);

    match delivery.validate(valid_input("urn:uuid:msg-1", 2)) {
        DeliveryValidationResult::Rejected(notice) => {
            assert_eq!(notice.code, DeliveryFailureCode::Replay);
        }
        DeliveryValidationResult::Accepted => panic!("expected replay rejection"),
    }
}

#[test]
fn integration_durable_guard_in_memory_store_round_trips_bundle() {
    let bundle = seeded_bundle();
    let mut store = InMemoryDurableGuardSnapshotStore::default();

    store
        .save_bundle(bundle.clone())
        .expect("save should succeed");

    assert_eq!(
        store.load_bundle().expect("load should succeed"),
        Some(bundle)
    );
}

#[test]
fn integration_durable_guard_file_store_round_trips_bundle_from_disk() {
    let path = temp_snapshot_path();
    let bundle = seeded_bundle();
    let mut store = FileDurableGuardSnapshotStore::new(path.clone()).expect("store should build");

    store
        .save_bundle(bundle.clone())
        .expect("save should succeed");

    assert_eq!(
        store.load_bundle().expect("load should succeed"),
        Some(bundle)
    );
    remove_if_present(&path);
}

#[test]
fn integration_durable_guard_store_invalid_schema_and_payload_fail_closed() {
    let mut store = InMemoryDurableGuardSnapshotStore::default();
    let mut bundle = seeded_bundle();
    bundle.schema_version = DURABLE_GUARD_BUNDLE_SCHEMA_VERSION + 1;
    assert_eq!(
        store.save_bundle(bundle),
        Err(
            DurableGuardSnapshotStoreError::BundleSchemaVersionMismatch {
                expected: DURABLE_GUARD_BUNDLE_SCHEMA_VERSION,
                found: DURABLE_GUARD_BUNDLE_SCHEMA_VERSION + 1,
            }
        )
    );

    let path = temp_snapshot_path();
    fs::write(&path, "not-json-or-legacy-payload").expect("payload write should succeed");
    let store = FileDurableGuardSnapshotStore::new(path.clone()).expect("store should build");
    assert!(matches!(
        store.load_bundle(),
        Err(DurableGuardSnapshotStoreError::InvalidPayload(_))
    ));
    remove_if_present(&path);
}
