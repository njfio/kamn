use super::policy_codec::{
    decode_hex, decode_permission_rule, decode_retention_policy, encode_hex,
    encode_permission_rule, encode_retention_policy,
};
use super::wire_codec::{deserialize_bundle, serialize_bundle};
use super::{
    DurableGuardBundleSnapshotStore, DurableGuardSnapshotBundle, DurableGuardSnapshotStoreError,
    InMemoryDurableGuardSnapshotStore,
};
use crate::{
    ChannelPermissionEngine, ChannelPermissions, DeliveryGuardInput, DeliveryValidationResult,
    MessageDeliveryGuards, PermissionRule, RetentionPolicy,
};
use std::collections::BTreeSet;

fn delivery_input(message_id: &str, nonce: u64, received_at: &str) -> DeliveryGuardInput {
    DeliveryGuardInput {
        message_id: message_id.to_owned(),
        sender: "kamn:did:agent:sender-1".to_owned(),
        recipient: "kamn:did:agent:recipient-1".to_owned(),
        nonce,
        created: "2026-02-09T00:00:00.000Z".to_owned(),
        expires: "2026-02-09T00:30:00.000Z".to_owned(),
        received_at: received_at.to_owned(),
    }
}

fn seeded_bundle() -> DurableGuardSnapshotBundle {
    let mut guards = MessageDeliveryGuards::new();
    let channels = seeded_channel_engine();
    assert_delivery_acceptance(&mut guards);
    DurableGuardSnapshotBundle::capture(&guards, &channels)
}

fn seeded_channel_engine() -> ChannelPermissionEngine {
    let mut channels = ChannelPermissionEngine::new();
    channels
        .register_channel(
            "channel:group:bundle-roundtrip",
            seeded_members(),
            vec!["kamn:did:agent:owner".to_owned()],
            seeded_permissions(),
        )
        .expect("channel registration should pass");
    channels
}

fn seeded_members() -> Vec<String> {
    vec![
        "kamn:did:agent:owner".to_owned(),
        "kamn:did:agent:member-1".to_owned(),
    ]
}

fn seeded_permissions() -> ChannelPermissions {
    ChannelPermissions {
        send: PermissionRule::Members,
        read: PermissionRule::Members,
        invite: PermissionRule::Admins,
        remove: PermissionRule::Admins,
        configure: PermissionRule::Admins,
        retention: RetentionPolicy::MaxMessageCount(2),
    }
}

fn assert_delivery_acceptance(guards: &mut MessageDeliveryGuards) {
    assert_eq!(
        guards.validate(delivery_input(
            "urn:uuid:bundle-roundtrip-1",
            1,
            "2026-02-09T00:10:00.000Z"
        )),
        DeliveryValidationResult::Accepted
    );
}

#[test]
fn hex_encoding_roundtrip() {
    let value = "kamn:did:agent:sender-1|nonce";
    let encoded = encode_hex(value);
    let decoded = decode_hex(&encoded).expect("hex decoding should pass");
    assert_eq!(decoded, value);
}

#[test]
fn permission_rule_encoding_roundtrip() {
    let rule = PermissionRule::Allowlist(BTreeSet::from([
        "kamn:did:agent:a".to_owned(),
        "kamn:did:agent:b".to_owned(),
    ]));
    let encoded = encode_permission_rule(&rule);
    let decoded = decode_permission_rule(&encoded).expect("rule decode should pass");
    assert_eq!(decoded, rule);
}

#[test]
fn retention_policy_encoding_roundtrip() {
    let policy = RetentionPolicy::MaxMessageCount(64);
    let encoded = encode_retention_policy(&policy);
    let decoded = decode_retention_policy(&encoded).expect("policy decode should pass");
    assert_eq!(decoded, policy);
}

#[test]
fn bundle_serialization_roundtrip() {
    let bundle = seeded_bundle();
    let payload = serialize_bundle(&bundle).expect("bundle serialization should pass");
    let decoded = deserialize_bundle(&payload).expect("bundle decode should pass");
    assert_eq!(decoded, bundle);
}

#[test]
fn regression_bundle_serialization_uses_json_payload() {
    let payload = serialize_bundle(&seeded_bundle()).expect("bundle serialization should pass");
    assert!(
        payload.trim_start().starts_with('{'),
        "expected serde JSON payload, found: {payload}"
    );
}

#[test]
fn in_memory_store_rejects_invalid_bundle_schema() {
    let mut bundle = seeded_bundle();
    bundle.schema_version = bundle.schema_version.saturating_add(1);
    let mut store = InMemoryDurableGuardSnapshotStore::default();
    assert_eq!(
        store.save_bundle(bundle),
        Err(
            DurableGuardSnapshotStoreError::BundleSchemaVersionMismatch {
                expected: 1,
                found: 2,
            }
        )
    );
}
