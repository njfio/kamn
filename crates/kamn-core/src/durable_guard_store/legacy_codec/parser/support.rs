use crate::{ChannelPolicySnapshot, ChannelPolicySnapshotChannel, DeliveryGuardSnapshot};
use std::collections::{BTreeMap, BTreeSet};

use crate::durable_guard_store::{validate_bundle, DurableGuardSnapshotBundle};
use crate::DurableGuardSnapshotStoreError;

pub(super) fn finish_bundle<'a, I>(
    lines: &mut I,
    bundle_schema: u16,
    delivery_schema: u16,
    channel_schema: u16,
    next_nonce_by_sender: BTreeMap<String, u64>,
    seen_message_ids: BTreeSet<String>,
    channels: Vec<ChannelPolicySnapshotChannel>,
) -> Result<DurableGuardSnapshotBundle, DurableGuardSnapshotStoreError>
where
    I: Iterator<Item = &'a str>,
{
    ensure_bundle_tail(lines)?;
    let bundle = build_bundle(
        bundle_schema,
        delivery_schema,
        channel_schema,
        next_nonce_by_sender,
        seen_message_ids,
        channels,
    );
    validate_bundle(&bundle)?;
    Ok(bundle)
}

fn ensure_bundle_tail<'a, I>(lines: &mut I) -> Result<(), DurableGuardSnapshotStoreError>
where
    I: Iterator<Item = &'a str>,
{
    let bundle_end = next_required_line(lines, "missing bundle_end marker")?;
    if bundle_end != "bundle_end|" {
        return invalid_payload(bundle_end);
    }
    if lines.next().is_some() {
        return invalid_payload("extra payload lines after bundle_end marker");
    }
    Ok(())
}

fn build_bundle(
    bundle_schema: u16,
    delivery_schema: u16,
    channel_schema: u16,
    next_nonce_by_sender: BTreeMap<String, u64>,
    seen_message_ids: BTreeSet<String>,
    channels: Vec<ChannelPolicySnapshotChannel>,
) -> DurableGuardSnapshotBundle {
    DurableGuardSnapshotBundle {
        schema_version: bundle_schema,
        delivery_guard: DeliveryGuardSnapshot {
            schema_version: delivery_schema,
            next_nonce_by_sender,
            seen_message_ids,
        },
        channel_policy: ChannelPolicySnapshot {
            schema_version: channel_schema,
            channels,
        },
    }
}

pub(super) fn parse_schema_line<'a, I>(
    lines: &mut I,
    prefix: &str,
    field_name: &'static str,
) -> Result<u16, DurableGuardSnapshotStoreError>
where
    I: Iterator<Item = &'a str>,
{
    let line = next_required_line(lines, &format!("missing {field_name} field"))?;
    let value = line.strip_prefix(prefix).ok_or_else(|| invalid_payload_err(line))?;
    value.parse::<u16>().map_err(|_| invalid_payload_err(line))
}

pub(super) fn next_required_line<'a, I>(
    lines: &mut I,
    message: &str,
) -> Result<&'a str, DurableGuardSnapshotStoreError>
where
    I: Iterator<Item = &'a str>,
{
    lines.next().ok_or_else(|| invalid_payload_err(message))
}

pub(super) fn invalid_payload<T>(
    value: &str,
) -> Result<T, DurableGuardSnapshotStoreError> {
    Err(invalid_payload_err(value))
}

pub(super) fn invalid_payload_err(value: &str) -> DurableGuardSnapshotStoreError {
    DurableGuardSnapshotStoreError::InvalidPayload(value.to_owned())
}
