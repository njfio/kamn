use kamn_core::{
    KeyLifecycle, KeyLifecycleAuditError, KeyLifecycleAuditRecord, KeyLifecycleError,
    KeyLifecycleEvent, KeyLifecycleState,
};

fn is_lower_hex(value: &str) -> bool {
    value
        .chars()
        .all(|char| char.is_ascii_digit() || ('a'..='f').contains(&char))
}

fn legacy_v0_record_hash(
    sequence: u64,
    event_kind: &str,
    event_payload: &str,
    previous_hash: &str,
) -> String {
    let canonical_payload = format!("{sequence}|{event_kind}|{event_payload}|{previous_hash}");
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in canonical_payload.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3_u64);
    }
    format!("{hash:016x}")
}

fn migrate_to_legacy_v0(records: &[KeyLifecycleAuditRecord]) -> Vec<KeyLifecycleAuditRecord> {
    let mut migrated = Vec::with_capacity(records.len());
    let mut previous_hash = "GENESIS".to_owned();

    for record in records {
        let record_hash = legacy_v0_record_hash(
            record.sequence,
            &record.event_kind,
            &record.event_payload,
            &previous_hash,
        );
        migrated.push(KeyLifecycleAuditRecord {
            sequence: record.sequence,
            event_kind: record.event_kind.clone(),
            event_payload: record.event_payload.clone(),
            previous_hash: previous_hash.clone(),
            record_hash: record_hash.clone(),
        });
        previous_hash = record_hash;
    }

    migrated
}

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

#[test]
fn spec_c01_issue_5925_audit_records_use_versioned_sha256_marker() {
    let mut lifecycle = KeyLifecycle::new("key_v1").expect("lifecycle should initialize");
    lifecycle
        .initiate_rotation("key_v2")
        .expect("rotation init should succeed");
    lifecycle
        .activate_rotation()
        .expect("rotation activation should succeed");

    let records = lifecycle.audit_records();
    assert!(!records.is_empty());
    for record in &records {
        assert!(record.record_hash.starts_with("sha256:v1:"));
        let digest = record.record_hash.trim_start_matches("sha256:v1:");
        assert_eq!(digest.len(), 64);
        assert!(is_lower_hex(digest));
    }
}

#[test]
fn spec_c02_issue_5925_collision_style_payload_mutation_changes_hash() {
    let mut lifecycle_a = KeyLifecycle::new("key_v1").expect("lifecycle should initialize");
    let mut lifecycle_b = KeyLifecycle::new("key_v1").expect("lifecycle should initialize");

    lifecycle_a
        .initiate_rotation("key_v2")
        .expect("rotation init should succeed");
    lifecycle_b
        .initiate_rotation("key_v2 ")
        .expect("rotation init should succeed");

    let hash_a = lifecycle_a.audit_records()[0].record_hash.clone();
    let hash_b = lifecycle_b.audit_records()[0].record_hash.clone();

    assert!(hash_a.starts_with("sha256:v1:"));
    assert!(hash_b.starts_with("sha256:v1:"));
    assert_ne!(hash_a, hash_b);
}

#[test]
fn spec_c03_issue_5925_legacy_v0_records_verify_for_migration() {
    let mut lifecycle = KeyLifecycle::new("key_v1").expect("lifecycle should initialize");
    lifecycle
        .initiate_rotation("key_v2")
        .expect("rotation init should succeed");
    lifecycle
        .activate_rotation()
        .expect("rotation activation should succeed");
    lifecycle.revoke().expect("revoke should succeed");

    let legacy_records = migrate_to_legacy_v0(&lifecycle.audit_records());
    assert!(!legacy_records[0].record_hash.starts_with("sha256:v1:"));

    KeyLifecycle::verify_audit_records(&legacy_records)
        .expect("legacy v0 records should verify for migration");
}

#[test]
fn regression_issue_5925_rejects_unknown_record_hash_format() {
    let mut lifecycle = KeyLifecycle::new("key_v1").expect("lifecycle should initialize");
    lifecycle
        .initiate_rotation("key_v2")
        .expect("rotation init should succeed");

    let mut records = lifecycle.audit_records();
    records[0].record_hash = "sha256:v1:not-a-valid-digest".to_owned();

    assert_eq!(
        KeyLifecycle::verify_audit_records(&records),
        Err(KeyLifecycleAuditError::HashMismatch { sequence: 1 })
    );
}
