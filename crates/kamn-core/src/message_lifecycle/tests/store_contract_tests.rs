use super::support::{advance_message, register_message, register_message_with_recipients};
use super::*;

#[test]
fn register_rejects_duplicate_id() {
    let mut store = MessageLifecycleStore::new();
    register_message(&mut store, "urn:uuid:msg-1");
    assert_eq!(
        store.register(
            "urn:uuid:msg-1",
            "kamn:did:agent:sender-1",
            vec!["kamn:did:agent:recipient-1".to_owned()],
            "2026-02-07T20:15:30.123Z",
            "2026-02-07T20:45:30.123Z",
        ),
        Err(MessageLifecycleError::DuplicateMessageId(
            "urn:uuid:msg-1".to_owned()
        ))
    );
}

#[test]
fn transition_updates_status_index() {
    let mut store = MessageLifecycleStore::new();
    register_message(&mut store, "urn:uuid:msg-2");
    advance_message(&mut store, "urn:uuid:msg-2", &[MessageStatus::Signed]);
    assert!(store.ids_by_status(MessageStatus::Created).is_empty());
    assert_eq!(
        store.ids_by_status(MessageStatus::Signed),
        vec!["urn:uuid:msg-2".to_owned()]
    );
}

#[test]
fn expire_message_if_overdue_rejects_empty_observed_timestamp() {
    let mut store = MessageLifecycleStore::new();
    register_message(&mut store, "urn:uuid:msg-2a");
    assert_eq!(
        store.expire_message_if_overdue("urn:uuid:msg-2a", " "),
        Err(MessageLifecycleError::EmptyTimestamp("observed_at"))
    );
}

#[test]
fn expire_overdue_messages_expires_created_and_validated_records() {
    let mut store = MessageLifecycleStore::new();
    register_message(&mut store, "urn:uuid:msg-2b");
    register_message(&mut store, "urn:uuid:msg-2c");
    advance_message(
        &mut store,
        "urn:uuid:msg-2c",
        &[
            MessageStatus::Signed,
            MessageStatus::Broadcast,
            MessageStatus::Included,
            MessageStatus::Delivered,
            MessageStatus::Validated,
        ],
    );
    let expired = store
        .expire_overdue_messages("2026-02-07T20:50:30.123Z")
        .expect("sweep should succeed");
    assert_eq!(
        expired,
        vec!["urn:uuid:msg-2b".to_owned(), "urn:uuid:msg-2c".to_owned()]
    );
    assert_eq!(
        store
            .status("urn:uuid:msg-2b")
            .expect("status should exist"),
        MessageStatus::Expired
    );
    assert_eq!(
        store
            .status("urn:uuid:msg-2c")
            .expect("status should exist"),
        MessageStatus::Expired
    );
}

#[test]
fn regression_issue_6194_expire_message_if_overdue_transitions_validated_message() {
    let mut store = MessageLifecycleStore::new();
    register_message(&mut store, "urn:uuid:msg-2d");
    advance_message(
        &mut store,
        "urn:uuid:msg-2d",
        &[
            MessageStatus::Signed,
            MessageStatus::Broadcast,
            MessageStatus::Included,
            MessageStatus::Delivered,
            MessageStatus::Validated,
        ],
    );
    assert!(store
        .expire_message_if_overdue("urn:uuid:msg-2d", "2026-02-07T20:50:30.123Z")
        .expect("validated message should expire once overdue"));
    assert_eq!(
        store
            .status("urn:uuid:msg-2d")
            .expect("status should exist"),
        MessageStatus::Expired
    );
}

#[test]
fn functional_message_lifecycle_snapshot_roundtrip_restores_indexes() {
    let mut store = MessageLifecycleStore::new();
    register_message_with_recipients(
        &mut store,
        "urn:uuid:msg-snapshot-1",
        vec![
            "kamn:did:agent:recipient-1".to_owned(),
            "kamn:did:agent:recipient-2".to_owned(),
        ],
    );
    advance_message(
        &mut store,
        "urn:uuid:msg-snapshot-1",
        &[MessageStatus::Signed],
    );
    let snapshot = store.export_snapshot();
    let mut restored = MessageLifecycleStore::new();
    restored
        .restore_snapshot(snapshot)
        .expect("snapshot restore should pass");
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
