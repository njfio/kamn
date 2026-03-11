use super::support::{register_message, snapshot_fixture};
use super::*;

#[test]
fn unit_parse_message_lifecycle_snapshot_payload_roundtrips_valid_payload() {
    let snapshot = snapshot_fixture(
        "urn:uuid:msg-parser-1",
        MessageStatus::Delivered,
        vec![
            MessageStatus::Created,
            MessageStatus::Signed,
            MessageStatus::Delivered,
        ],
    );
    let payload =
        serialize_message_lifecycle_snapshot(&snapshot).expect("snapshot should serialize");
    assert_eq!(
        parse_message_lifecycle_snapshot_payload(&payload).expect("payload should parse"),
        snapshot
    );
}

#[test]
fn regression_parse_message_lifecycle_snapshot_payload_rejects_malformed_schema_line() {
    assert_eq!(
        parse_message_lifecycle_snapshot_payload("schema\nrecord|broken"),
        Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
            "schema".to_owned()
        ))
    );
}

#[test]
fn regression_parse_message_lifecycle_snapshot_payload_rejects_malformed_record_field_count() {
    assert_eq!(
        parse_message_lifecycle_snapshot_payload("schema|1\nrecord|broken"),
        Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
            "record|broken".to_owned()
        ))
    );
}

#[test]
fn regression_parse_message_lifecycle_snapshot_payload_rejects_invalid_status_and_history_codes() {
    assert_eq!(
        parse_message_lifecycle_snapshot_payload(
            "schema|1\nrecord|urn:uuid:msg|sender|recipient|created|expires|99|0"
        ),
        Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
            "record|urn:uuid:msg|sender|recipient|created|expires|99|0".to_owned()
        ))
    );
    assert_eq!(
        parse_message_lifecycle_snapshot_payload(
            "schema|1\nrecord|urn:uuid:msg|sender|recipient|created|expires|1|0,99"
        ),
        Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
            "record|urn:uuid:msg|sender|recipient|created|expires|1|0,99".to_owned()
        ))
    );
}

#[test]
fn regression_message_lifecycle_snapshot_restore_rejects_duplicate_message_ids() {
    let mut store = MessageLifecycleStore::new();
    register_message(&mut store, "urn:uuid:msg-snapshot-2");
    let mut snapshot = store.export_snapshot();
    snapshot.records.push(snapshot.records[0].clone());
    let mut restored = MessageLifecycleStore::new();
    assert_eq!(
        restored.restore_snapshot(snapshot),
        Err(MessageLifecycleSnapshotError::DuplicateMessageId(
            "urn:uuid:msg-snapshot-2".to_owned()
        ))
    );
}

#[test]
fn regression_message_lifecycle_snapshot_restore_rejects_status_history_mismatch() {
    let snapshot = snapshot_fixture(
        "urn:uuid:msg-snapshot-3",
        MessageStatus::Delivered,
        vec![MessageStatus::Created, MessageStatus::Signed],
    );
    let mut restored = MessageLifecycleStore::new();
    assert_eq!(
        restored.restore_snapshot(snapshot),
        Err(MessageLifecycleSnapshotError::InvalidSnapshot(
            "status/history mismatch for urn:uuid:msg-snapshot-3".to_owned()
        ))
    );
}
