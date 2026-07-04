use super::wire_types::{
    ChannelPermissionsWire, ChannelPolicySnapshotChannelWire, ChannelPolicySnapshotWire,
    DeliveryGuardSnapshotWire, DeliveryNonceWire, DurableGuardSnapshotBundleWire,
};
use crate::{
    ChannelPermissions, ChannelPolicySnapshot, ChannelPolicySnapshotChannel, DeliveryGuardSnapshot,
};
use std::collections::{BTreeMap, BTreeSet};

use super::super::legacy_codec::deserialize_bundle_legacy;
use super::super::policy_codec::{
    decode_hex, decode_permission_rule, decode_retention_policy, encode_hex,
    encode_permission_rule, encode_retention_policy,
};
use super::super::{validate_bundle, DurableGuardSnapshotBundle, DurableGuardSnapshotStoreError};

pub(crate) fn serialize_bundle(
    bundle: &DurableGuardSnapshotBundle,
) -> Result<String, DurableGuardSnapshotStoreError> {
    let wire = DurableGuardSnapshotBundleWire {
        schema_version: bundle.schema_version,
        delivery_guard: encode_delivery_guard(&bundle.delivery_guard),
        channel_policy: encode_channel_policy(&bundle.channel_policy),
    };
    serde_json::to_string(&wire)
        .map_err(|error| DurableGuardSnapshotStoreError::InvalidPayload(error.to_string()))
}

pub(crate) fn deserialize_bundle(
    payload: &str,
) -> Result<DurableGuardSnapshotBundle, DurableGuardSnapshotStoreError> {
    match serde_json::from_str::<DurableGuardSnapshotBundleWire>(payload) {
        Ok(wire) => build_bundle_from_wire(wire),
        Err(_) => deserialize_bundle_legacy(payload),
    }
}

fn encode_delivery_guard(snapshot: &DeliveryGuardSnapshot) -> DeliveryGuardSnapshotWire {
    DeliveryGuardSnapshotWire {
        schema_version: snapshot.schema_version,
        next_nonce_by_sender: snapshot
            .next_nonce_by_sender
            .iter()
            .map(|(sender, nonce)| DeliveryNonceWire {
                sender: encode_hex(sender),
                nonce: *nonce,
            })
            .collect(),
        seen_message_ids: snapshot
            .seen_message_ids
            .iter()
            .map(|message_id| encode_hex(message_id))
            .collect(),
    }
}

fn encode_channel_policy(snapshot: &ChannelPolicySnapshot) -> ChannelPolicySnapshotWire {
    ChannelPolicySnapshotWire {
        schema_version: snapshot.schema_version,
        channels: snapshot.channels.iter().map(encode_channel).collect(),
    }
}

fn encode_channel(channel: &ChannelPolicySnapshotChannel) -> ChannelPolicySnapshotChannelWire {
    ChannelPolicySnapshotChannelWire {
        channel_id: encode_hex(&channel.channel_id),
        members: channel
            .members
            .iter()
            .map(|member| encode_hex(member))
            .collect(),
        admins: channel
            .admins
            .iter()
            .map(|admin| encode_hex(admin))
            .collect(),
        permissions: ChannelPermissionsWire {
            send: encode_permission_rule(&channel.permissions.send),
            read: encode_permission_rule(&channel.permissions.read),
            invite: encode_permission_rule(&channel.permissions.invite),
            remove: encode_permission_rule(&channel.permissions.remove),
            configure: encode_permission_rule(&channel.permissions.configure),
            retention: encode_retention_policy(&channel.permissions.retention),
        },
    }
}

fn build_bundle_from_wire(
    wire: DurableGuardSnapshotBundleWire,
) -> Result<DurableGuardSnapshotBundle, DurableGuardSnapshotStoreError> {
    let bundle = DurableGuardSnapshotBundle {
        schema_version: wire.schema_version,
        delivery_guard: DeliveryGuardSnapshot {
            schema_version: wire.delivery_guard.schema_version,
            next_nonce_by_sender: decode_nonce_map(wire.delivery_guard.next_nonce_by_sender)?,
            seen_message_ids: decode_seen_ids(wire.delivery_guard.seen_message_ids)?,
        },
        channel_policy: ChannelPolicySnapshot {
            schema_version: wire.channel_policy.schema_version,
            channels: wire
                .channel_policy
                .channels
                .into_iter()
                .map(decode_channel)
                .collect::<Result<Vec<_>, _>>()?,
        },
    };
    validate_bundle(&bundle)?;
    Ok(bundle)
}

fn decode_nonce_map(
    entries: Vec<DeliveryNonceWire>,
) -> Result<BTreeMap<String, u64>, DurableGuardSnapshotStoreError> {
    let mut next_nonce_by_sender = BTreeMap::new();
    for entry in entries {
        let sender = decode_hex(entry.sender.as_str())?;
        if next_nonce_by_sender.insert(sender, entry.nonce).is_some() {
            return Err(DurableGuardSnapshotStoreError::InvalidPayload(
                "duplicate sender nonce record".to_owned(),
            ));
        }
    }
    Ok(next_nonce_by_sender)
}

fn decode_seen_ids(
    values: Vec<String>,
) -> Result<BTreeSet<String>, DurableGuardSnapshotStoreError> {
    let mut seen_message_ids = BTreeSet::new();
    for value in values {
        let message_id = decode_hex(value.as_str())?;
        if !seen_message_ids.insert(message_id) {
            return Err(DurableGuardSnapshotStoreError::InvalidPayload(
                "duplicate seen message record".to_owned(),
            ));
        }
    }
    Ok(seen_message_ids)
}

fn decode_channel(
    channel: ChannelPolicySnapshotChannelWire,
) -> Result<ChannelPolicySnapshotChannel, DurableGuardSnapshotStoreError> {
    Ok(ChannelPolicySnapshotChannel {
        channel_id: decode_hex(channel.channel_id.as_str())?,
        members: decode_string_vec(channel.members)?,
        admins: decode_string_vec(channel.admins)?,
        permissions: ChannelPermissions {
            send: decode_permission_rule(channel.permissions.send.as_str())?,
            read: decode_permission_rule(channel.permissions.read.as_str())?,
            invite: decode_permission_rule(channel.permissions.invite.as_str())?,
            remove: decode_permission_rule(channel.permissions.remove.as_str())?,
            configure: decode_permission_rule(channel.permissions.configure.as_str())?,
            retention: decode_retention_policy(channel.permissions.retention.as_str())?,
        },
    })
}

fn decode_string_vec(values: Vec<String>) -> Result<Vec<String>, DurableGuardSnapshotStoreError> {
    values
        .into_iter()
        .map(|value| decode_hex(value.as_str()))
        .collect()
}
