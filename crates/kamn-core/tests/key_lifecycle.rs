use kamn_core::{
    KeyLifecycle, KeyLifecycleAuditError, KeyLifecycleError, KeyLifecycleEvent, KeyLifecycleState,
};

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

#[test]
fn audit_records_form_tamper_evident_chain() {
    let mut lifecycle = KeyLifecycle::new("key_v1").expect("lifecycle should initialize");
    lifecycle
        .initiate_rotation("key_v2")
        .expect("rotation init should succeed");
    lifecycle
        .activate_rotation()
        .expect("rotation activation should succeed");
    lifecycle.revoke().expect("revoke should succeed");

    let records = lifecycle.audit_records();
    assert_eq!(records.len(), 3);
    assert_eq!(records[0].previous_hash, "GENESIS");
    assert_eq!(records[0].sequence, 1);
    assert_eq!(records[1].sequence, 2);
    assert_eq!(records[2].sequence, 3);
    assert_ne!(records[0].record_hash, records[1].record_hash);

    lifecycle
        .verify_audit_trail()
        .expect("audit trail should verify");
}

#[test]
fn verify_rejects_tampered_hash_chain() {
    let mut lifecycle = KeyLifecycle::new("key_v1").expect("lifecycle should initialize");
    lifecycle
        .initiate_rotation("key_v2")
        .expect("rotation init should succeed");
    lifecycle
        .activate_rotation()
        .expect("rotation activation should succeed");

    let mut records = lifecycle.audit_records();
    records[1].previous_hash = "tampered-link".to_owned();

    assert_eq!(
        KeyLifecycle::verify_audit_records(&records),
        Err(KeyLifecycleAuditError::BrokenHashChain { sequence: 2 })
    );
}

#[test]
fn regression_detects_sequence_gap_in_audit_records() {
    let mut lifecycle = KeyLifecycle::new("key_v1").expect("lifecycle should initialize");
    lifecycle
        .initiate_rotation("key_v2")
        .expect("rotation init should succeed");
    lifecycle
        .activate_rotation()
        .expect("rotation activation should succeed");

    let mut records = lifecycle.audit_records();
    records[1].sequence = 99;

    assert_eq!(
        KeyLifecycle::verify_audit_records(&records),
        Err(KeyLifecycleAuditError::SequenceGap {
            expected: 2,
            found: 99,
        })
    );
}
