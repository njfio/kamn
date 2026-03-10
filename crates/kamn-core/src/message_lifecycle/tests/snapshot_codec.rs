#[test]
fn functional_message_lifecycle_snapshot_roundtrip_restores_indexes() {
    let snapshot = signed_snapshot_with_two_recipients();
    let mut restored = MessageLifecycleStore::new();
    restored
        .restore_snapshot(snapshot)
        .expect("snapshot restore should pass");
    assert_signed_snapshot_indexes(&restored);
}

#[test]
fn unit_parse_message_lifecycle_snapshot_payload_roundtrips_valid_payload() {
    let snapshot = roundtrip_parser_snapshot();
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
    // Regression: #617
    let mut store = MessageLifecycleStore::new();
    register_default_message(&mut store, "urn:uuid:msg-snapshot-2");
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
    // Regression: #617
    let snapshot = MessageLifecycleSnapshot {
        schema_version: 1,
        records: vec![sample_snapshot_record(
            "urn:uuid:msg-snapshot-3",
            MessageStatus::Delivered,
            vec![MessageStatus::Created, MessageStatus::Signed],
        )],
    };

    let mut restored = MessageLifecycleStore::new();
    assert_eq!(
        restored.restore_snapshot(snapshot),
        Err(MessageLifecycleSnapshotError::InvalidSnapshot(
            "status/history mismatch for urn:uuid:msg-snapshot-3".to_owned()
        ))
    );
}

fn signed_snapshot_with_two_recipients() -> MessageLifecycleSnapshot {
    let mut store = MessageLifecycleStore::new();
    register_message(
        &mut store,
        "urn:uuid:msg-snapshot-1",
        "kamn:did:agent:sender-1",
        vec![
            "kamn:did:agent:recipient-1".to_owned(),
            "kamn:did:agent:recipient-2".to_owned(),
        ],
    );
    store
        .transition("urn:uuid:msg-snapshot-1", MessageStatus::Signed)
        .expect("created->signed should succeed");
    store.export_snapshot()
}

fn assert_signed_snapshot_indexes(restored: &MessageLifecycleStore) {
    assert_eq!(
        restored
            .status("urn:uuid:msg-snapshot-1")
            .expect("status should exist"),
        MessageStatus::Signed
    );
    assert_eq!(
        restored.ids_by_sender("kamn:did:agent:sender-1"),
        vec!["urn:uuid:msg-snapshot-1".to_owned()]
    );
    assert_eq!(
        restored.ids_by_recipient("kamn:did:agent:recipient-1"),
        vec!["urn:uuid:msg-snapshot-1".to_owned()]
    );
}

fn roundtrip_parser_snapshot() -> MessageLifecycleSnapshot {
    MessageLifecycleSnapshot {
        schema_version: 1,
        records: vec![MessageRecordSnapshot {
            recipients: vec![
                "kamn:did:agent:recipient-1".to_owned(),
                "kamn:did:agent:recipient-2".to_owned(),
            ],
            ..sample_snapshot_record(
                "urn:uuid:msg-parser-1",
                MessageStatus::Delivered,
                vec![
                    MessageStatus::Created,
                    MessageStatus::Signed,
                    MessageStatus::Delivered,
                ],
            )
        }],
    }
}
