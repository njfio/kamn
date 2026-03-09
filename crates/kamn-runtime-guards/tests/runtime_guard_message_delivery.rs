use std::collections::{BTreeMap, BTreeSet};

use kamn_runtime_guards::message_delivery_guards::{
    DeliveryFailureCode, DeliveryGuardInput, DeliveryGuardSnapshot, DeliveryGuardSnapshotError,
    DeliveryValidationResult, MessageDeliveryGuards, DELIVERY_GUARD_SNAPSHOT_SCHEMA_VERSION,
};

fn input(message_id: &str, nonce: u64, received_at: &str) -> DeliveryGuardInput {
    DeliveryGuardInput {
        message_id: message_id.to_owned(),
        sender: "kamn:did:agent:sender-1".to_owned(),
        recipient: "kamn:did:agent:recipient-1".to_owned(),
        nonce,
        created: "2026-02-07T20:15:30.123Z".to_owned(),
        expires: "2026-02-07T20:45:30.123Z".to_owned(),
        received_at: received_at.to_owned(),
    }
}

fn assert_rejected(
    result: DeliveryValidationResult,
) -> kamn_runtime_guards::message_delivery_guards::FailedDeliveryNotice {
    match result {
        DeliveryValidationResult::Rejected(notice) => notice,
        DeliveryValidationResult::Accepted => panic!("expected rejection"),
    }
}

#[test]
fn integration_runtime_guard_message_delivery_accepts_first_message_and_advances_nonce() {
    let mut guards = MessageDeliveryGuards::new();

    assert_eq!(
        guards.validate(input("urn:uuid:msg-1", 1, "2026-02-07T20:20:30.123Z")),
        DeliveryValidationResult::Accepted
    );
    assert_eq!(guards.expected_nonce("kamn:did:agent:sender-1"), 2);
}

#[test]
fn integration_runtime_guard_message_delivery_rejects_replay_and_nonce_regression() {
    let mut guards = MessageDeliveryGuards::new();
    assert_eq!(
        guards.validate(input("urn:uuid:msg-2", 1, "2026-02-07T20:20:30.123Z")),
        DeliveryValidationResult::Accepted
    );

    let replay_notice =
        assert_rejected(guards.validate(input("urn:uuid:msg-2", 2, "2026-02-07T20:21:30.123Z")));
    assert_eq!(replay_notice.code, DeliveryFailureCode::Replay);
    assert_eq!(
        replay_notice.signature,
        "notice:urn:uuid:msg-2:replay:kamn:did:agent:recipient-1:2026-02-07T20:21:30.123Z:2"
    );

    let nonce_notice =
        assert_rejected(guards.validate(input("urn:uuid:msg-3", 1, "2026-02-07T20:22:30.123Z")));
    assert_eq!(
        nonce_notice.code,
        DeliveryFailureCode::NonceOutOfSequence {
            expected: 2,
            found: 1,
        }
    );
    assert_eq!(
        nonce_notice.signature,
        "notice:urn:uuid:msg-3:nonce_out_of_sequence:kamn:did:agent:recipient-1:2026-02-07T20:22:30.123Z:1"
    );
}

#[test]
fn integration_runtime_guard_message_delivery_snapshot_roundtrip_restores_replay_state() {
    let mut guards = MessageDeliveryGuards::new();
    assert_eq!(
        guards.validate(input("urn:uuid:msg-4", 1, "2026-02-07T20:20:30.123Z")),
        DeliveryValidationResult::Accepted
    );

    let restored = MessageDeliveryGuards::from_snapshot(guards.export_snapshot())
        .expect("snapshot restore should succeed");
    assert_eq!(restored.expected_nonce("kamn:did:agent:sender-1"), 2);

    let mut restored_mut = restored;
    let replay_notice = assert_rejected(restored_mut.validate(input(
        "urn:uuid:msg-4",
        2,
        "2026-02-07T20:21:30.123Z",
    )));
    assert_eq!(replay_notice.code, DeliveryFailureCode::Replay);
}

#[test]
fn integration_runtime_guard_message_delivery_invalid_snapshot_fails_closed() {
    assert_eq!(
        MessageDeliveryGuards::from_snapshot(DeliveryGuardSnapshot {
            schema_version: DELIVERY_GUARD_SNAPSHOT_SCHEMA_VERSION + 1,
            next_nonce_by_sender: BTreeMap::new(),
            seen_message_ids: BTreeSet::new(),
        }),
        Err(DeliveryGuardSnapshotError::SchemaVersionMismatch {
            expected: DELIVERY_GUARD_SNAPSHOT_SCHEMA_VERSION,
            found: DELIVERY_GUARD_SNAPSHOT_SCHEMA_VERSION + 1,
        })
    );
    assert_eq!(
        MessageDeliveryGuards::from_snapshot(DeliveryGuardSnapshot {
            schema_version: DELIVERY_GUARD_SNAPSHOT_SCHEMA_VERSION,
            next_nonce_by_sender: BTreeMap::from([(" ".to_owned(), 1)]),
            seen_message_ids: BTreeSet::new(),
        }),
        Err(DeliveryGuardSnapshotError::InvalidSender(" ".to_owned()))
    );
    assert_eq!(
        MessageDeliveryGuards::from_snapshot(DeliveryGuardSnapshot {
            schema_version: DELIVERY_GUARD_SNAPSHOT_SCHEMA_VERSION,
            next_nonce_by_sender: BTreeMap::from([("kamn:did:agent:sender-1".to_owned(), 0,)]),
            seen_message_ids: BTreeSet::new(),
        }),
        Err(DeliveryGuardSnapshotError::InvalidNonce {
            sender: "kamn:did:agent:sender-1".to_owned(),
            nonce: 0,
        })
    );
    assert_eq!(
        MessageDeliveryGuards::from_snapshot(DeliveryGuardSnapshot {
            schema_version: DELIVERY_GUARD_SNAPSHOT_SCHEMA_VERSION,
            next_nonce_by_sender: BTreeMap::new(),
            seen_message_ids: BTreeSet::from([" ".to_owned()]),
        }),
        Err(DeliveryGuardSnapshotError::InvalidMessageId(" ".to_owned()))
    );
}
