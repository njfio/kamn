use super::super::channel_policy_builder::ChannelPolicyBuilder;
use super::support::{invalid_payload, invalid_payload_err, next_required_line};
use crate::durable_guard_store::policy_codec::{
    decode_hex, decode_permission_rule, decode_retention_policy,
};
use crate::{ChannelPolicySnapshotChannel, DurableGuardSnapshotStoreError};

pub(super) fn parse_channel_section<'a, I>(
    lines: &mut I,
) -> Result<Vec<ChannelPolicySnapshotChannel>, DurableGuardSnapshotStoreError>
where
    I: Iterator<Item = &'a str>,
{
    let mut channels = Vec::new();
    let mut builder: Option<ChannelPolicyBuilder> = None;
    loop {
        let line = next_required_line(lines, "missing channel_end_all marker")?;
        if line == "channel_end_all|" {
            ensure_no_open_channel(builder)?;
            return Ok(channels);
        }
        handle_channel_line(line, &mut builder, &mut channels)?;
    }
}

fn handle_channel_line(
    line: &str,
    builder: &mut Option<ChannelPolicyBuilder>,
    channels: &mut Vec<ChannelPolicySnapshotChannel>,
) -> Result<(), DurableGuardSnapshotStoreError> {
    if let Some(value) = line.strip_prefix("channel_begin|") {
        *builder = Some(begin_channel(builder.take(), value)?);
        return Ok(());
    }
    if line == "channel_end|" {
        let value = builder
            .take()
            .ok_or_else(|| invalid_payload_err("channel_end marker without channel_begin"))?;
        channels.push(value.build()?);
        return Ok(());
    }
    let active = builder
        .as_mut()
        .ok_or_else(|| invalid_payload_err("channel field found without channel_begin"))?;
    parse_channel_field(line, active)
}

fn begin_channel(
    existing: Option<ChannelPolicyBuilder>,
    value: &str,
) -> Result<ChannelPolicyBuilder, DurableGuardSnapshotStoreError> {
    if existing.is_some() {
        return invalid_payload("nested channel_begin marker");
    }
    Ok(ChannelPolicyBuilder::new(decode_hex(value)?))
}

fn parse_channel_field(
    line: &str,
    active: &mut ChannelPolicyBuilder,
) -> Result<(), DurableGuardSnapshotStoreError> {
    if let Some(value) = line.strip_prefix("channel_member|") {
        active.members_mut().push(decode_hex(value)?);
        return Ok(());
    }
    if let Some(value) = line.strip_prefix("channel_admin|") {
        active.admins_mut().push(decode_hex(value)?);
        return Ok(());
    }
    parse_channel_permission_field(line, active)
}

fn parse_channel_permission_field(
    line: &str,
    active: &mut ChannelPolicyBuilder,
) -> Result<(), DurableGuardSnapshotStoreError> {
    if let Some(value) = line.strip_prefix("channel_perm_send|") {
        return active.set_send(decode_permission_rule(value)?);
    }
    if let Some(value) = line.strip_prefix("channel_perm_read|") {
        return active.set_read(decode_permission_rule(value)?);
    }
    if let Some(value) = line.strip_prefix("channel_perm_invite|") {
        return active.set_invite(decode_permission_rule(value)?);
    }
    if let Some(value) = line.strip_prefix("channel_perm_remove|") {
        return active.set_remove(decode_permission_rule(value)?);
    }
    if let Some(value) = line.strip_prefix("channel_perm_configure|") {
        return active.set_configure(decode_permission_rule(value)?);
    }
    if let Some(value) = line.strip_prefix("channel_retention|") {
        return active.set_retention(decode_retention_policy(value)?);
    }
    invalid_payload(line)
}

fn ensure_no_open_channel(
    builder: Option<ChannelPolicyBuilder>,
) -> Result<(), DurableGuardSnapshotStoreError> {
    if builder.is_some() {
        return invalid_payload("unterminated channel block");
    }
    Ok(())
}
