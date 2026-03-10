#[test]
fn register_rejects_duplicate_id() {
    let mut store = MessageLifecycleStore::new();
    register_default_message(&mut store, "urn:uuid:msg-1");

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
    register_default_message(&mut store, "urn:uuid:msg-2");

    store
        .transition("urn:uuid:msg-2", MessageStatus::Signed)
        .expect("created->signed should succeed");
    assert!(store.ids_by_status(MessageStatus::Created).is_empty());
    assert_eq!(
        store.ids_by_status(MessageStatus::Signed),
        vec!["urn:uuid:msg-2".to_owned()]
    );
}

#[test]
fn expire_message_if_overdue_rejects_empty_observed_timestamp() {
    let mut store = MessageLifecycleStore::new();
    register_default_message(&mut store, "urn:uuid:msg-2a");

    assert_eq!(
        store.expire_message_if_overdue("urn:uuid:msg-2a", " "),
        Err(MessageLifecycleError::EmptyTimestamp("observed_at"))
    );
}

#[test]
fn expire_overdue_messages_expires_created_and_validated_records() {
    let mut store = MessageLifecycleStore::new();
    register_default_message(&mut store, "urn:uuid:msg-2b");
    register_default_message(&mut store, "urn:uuid:msg-2c");
    transition_to_validated(&mut store, "urn:uuid:msg-2c");

    assert_eq!(
        store
            .expire_overdue_messages("2026-02-07T20:50:30.123Z")
            .expect("sweep should succeed"),
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
    register_default_message(&mut store, "urn:uuid:msg-2d");
    transition_to_validated(&mut store, "urn:uuid:msg-2d");

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
