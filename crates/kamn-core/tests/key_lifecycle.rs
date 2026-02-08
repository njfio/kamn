use kamn_core::{KeyLifecycle, KeyLifecycleError, KeyLifecycleEvent, KeyLifecycleState};

#[test]
fn active_to_rotating_to_active_emits_audit_events() {
    let mut lifecycle = KeyLifecycle::new("key_v1").expect("lifecycle should initialize");
    assert_eq!(lifecycle.state(), KeyLifecycleState::Active);
    assert_eq!(lifecycle.active_key_id(), "key_v1");

    lifecycle
        .initiate_rotation("key_v2")
        .expect("rotation init should succeed");
    assert_eq!(lifecycle.state(), KeyLifecycleState::Rotating);
    assert_eq!(lifecycle.pending_key_id(), Some("key_v2"));

    lifecycle
        .activate_rotation()
        .expect("rotation activation should succeed");
    assert_eq!(lifecycle.state(), KeyLifecycleState::Active);
    assert_eq!(lifecycle.active_key_id(), "key_v2");
    assert_eq!(lifecycle.pending_key_id(), None);

    assert_eq!(
        lifecycle.events(),
        &[
            KeyLifecycleEvent::RotationInitiated {
                sequence: 1,
                from_key: "key_v1".to_owned(),
                to_key: "key_v2".to_owned(),
            },
            KeyLifecycleEvent::RotationActivated {
                sequence: 2,
                active_key: "key_v2".to_owned(),
            },
        ]
    );
}

#[test]
fn activate_without_pending_rotation_is_rejected() {
    let mut lifecycle = KeyLifecycle::new("key_v1").expect("lifecycle should initialize");

    assert_eq!(
        lifecycle.activate_rotation(),
        Err(KeyLifecycleError::InvalidTransition {
            from: KeyLifecycleState::Active,
            action: "activate_rotation",
        })
    );
}

#[test]
fn revoke_blocks_future_rotation() {
    let mut lifecycle = KeyLifecycle::new("key_v1").expect("lifecycle should initialize");
    lifecycle.revoke().expect("revoke should succeed");
    assert_eq!(lifecycle.state(), KeyLifecycleState::Revoked);

    assert_eq!(
        lifecycle.initiate_rotation("key_v2"),
        Err(KeyLifecycleError::InvalidTransition {
            from: KeyLifecycleState::Revoked,
            action: "initiate_rotation",
        })
    );
}

#[test]
fn rotation_rejects_same_key_or_empty_key() {
    let mut lifecycle = KeyLifecycle::new("key_v1").expect("lifecycle should initialize");
    assert_eq!(
        lifecycle.initiate_rotation(""),
        Err(KeyLifecycleError::EmptyKeyId)
    );
    assert_eq!(
        lifecycle.initiate_rotation("key_v1"),
        Err(KeyLifecycleError::RotationKeyUnchanged)
    );
}
