use kamn_core::{MessageLifecycleError, MessageLifecycleStore, MessageStatus};

fn register_baseline(store: &mut MessageLifecycleStore, message_id: &str) {
    store
        .register(
            message_id,
            "kamn:did:agent:sender-1",
            vec![
                "kamn:did:agent:recipient-1".to_owned(),
                "kamn:did:agent:recipient-2".to_owned(),
            ],
            "2026-02-07T20:15:30.123Z",
            "2026-02-07T20:45:30.123Z",
        )
        .expect("register should succeed");
}

#[test]
fn register_starts_message_in_created_and_indexes_participants() {
    let mut store = MessageLifecycleStore::new();
    register_baseline(&mut store, "urn:uuid:msg-1");

    assert_eq!(
        store.status("urn:uuid:msg-1").expect("status should exist"),
        MessageStatus::Created
    );
    assert_eq!(
        store.ids_by_sender("kamn:did:agent:sender-1"),
        vec!["urn:uuid:msg-1".to_owned()]
    );
    assert_eq!(
        store.ids_by_recipient("kamn:did:agent:recipient-1"),
        vec!["urn:uuid:msg-1".to_owned()]
    );
}

#[test]
fn valid_lifecycle_transitions_are_indexed() {
    let mut store = MessageLifecycleStore::new();
    register_baseline(&mut store, "urn:uuid:msg-2");

    store
        .transition("urn:uuid:msg-2", MessageStatus::Signed)
        .expect("created->signed should succeed");
    store
        .transition("urn:uuid:msg-2", MessageStatus::Broadcast)
        .expect("signed->broadcast should succeed");
    store
        .transition("urn:uuid:msg-2", MessageStatus::Included)
        .expect("broadcast->included should succeed");
    store
        .transition("urn:uuid:msg-2", MessageStatus::Delivered)
        .expect("included->delivered should succeed");
    store
        .transition("urn:uuid:msg-2", MessageStatus::Validated)
        .expect("delivered->validated should succeed");

    assert_eq!(
        store.ids_by_status(MessageStatus::Validated),
        vec!["urn:uuid:msg-2".to_owned()]
    );
}

#[test]
fn invalid_transition_is_rejected() {
    let mut store = MessageLifecycleStore::new();
    register_baseline(&mut store, "urn:uuid:msg-3");

    assert_eq!(
        store.transition("urn:uuid:msg-3", MessageStatus::Broadcast),
        Err(MessageLifecycleError::InvalidTransition {
            from: MessageStatus::Created,
            to: MessageStatus::Broadcast,
        })
    );
}

#[test]
fn transition_rejects_unknown_message() {
    let mut store = MessageLifecycleStore::new();
    assert_eq!(
        store.transition("urn:uuid:missing", MessageStatus::Signed),
        Err(MessageLifecycleError::NotFound(
            "urn:uuid:missing".to_owned()
        ))
    );
}

#[test]
fn expired_message_cannot_reenter_pipeline() {
    let mut store = MessageLifecycleStore::new();
    register_baseline(&mut store, "urn:uuid:msg-4");
    store
        .transition("urn:uuid:msg-4", MessageStatus::Signed)
        .expect("created->signed should succeed");
    store
        .transition("urn:uuid:msg-4", MessageStatus::Broadcast)
        .expect("signed->broadcast should succeed");
    store
        .transition("urn:uuid:msg-4", MessageStatus::Included)
        .expect("broadcast->included should succeed");
    store
        .transition("urn:uuid:msg-4", MessageStatus::Delivered)
        .expect("included->delivered should succeed");
    store
        .transition("urn:uuid:msg-4", MessageStatus::Validated)
        .expect("delivered->validated should succeed");
    store
        .transition("urn:uuid:msg-4", MessageStatus::Rejected)
        .expect("validated->rejected should succeed");
    store
        .transition("urn:uuid:msg-4", MessageStatus::Expired)
        .expect("rejected->expired should succeed");

    // Regression: #115
    assert_eq!(
        store.transition("urn:uuid:msg-4", MessageStatus::Delivered),
        Err(MessageLifecycleError::InvalidTransition {
            from: MessageStatus::Expired,
            to: MessageStatus::Delivered,
        })
    );
}
