use std::collections::{BTreeMap, BTreeSet};

use super::support::{invalid_payload, invalid_payload_err, next_required_line};
use crate::durable_guard_store::policy_codec::decode_hex;
use crate::DurableGuardSnapshotStoreError;

pub(super) fn parse_delivery_section<'a, I>(
    lines: &mut I,
) -> Result<(BTreeMap<String, u64>, BTreeSet<String>), DurableGuardSnapshotStoreError>
where
    I: Iterator<Item = &'a str>,
{
    let mut next_nonce_by_sender = BTreeMap::new();
    let mut seen_message_ids = BTreeSet::new();
    loop {
        let line = next_required_line(lines, "missing delivery_end marker")?;
        if line == "delivery_end|" {
            return Ok((next_nonce_by_sender, seen_message_ids));
        }
        parse_delivery_line(line, &mut next_nonce_by_sender, &mut seen_message_ids)?;
    }
}

fn parse_delivery_line(
    line: &str,
    next_nonce_by_sender: &mut BTreeMap<String, u64>,
    seen_message_ids: &mut BTreeSet<String>,
) -> Result<(), DurableGuardSnapshotStoreError> {
    if let Some(value) = line.strip_prefix("delivery_nonce|") {
        return parse_delivery_nonce(value, line, next_nonce_by_sender);
    }
    if let Some(value) = line.strip_prefix("delivery_seen|") {
        let message_id = decode_hex(value)?;
        if !seen_message_ids.insert(message_id) {
            return invalid_payload(line);
        }
        return Ok(());
    }
    invalid_payload(line)
}

fn parse_delivery_nonce(
    value: &str,
    line: &str,
    next_nonce_by_sender: &mut BTreeMap<String, u64>,
) -> Result<(), DurableGuardSnapshotStoreError> {
    let mut parts = value.splitn(2, '|');
    let sender_hex = parts.next().ok_or_else(|| invalid_payload_err(line))?;
    let nonce_raw = parts.next().ok_or_else(|| invalid_payload_err(line))?;
    let sender = decode_hex(sender_hex)?;
    let nonce = nonce_raw
        .parse::<u64>()
        .map_err(|_| invalid_payload_err(line))?;
    if next_nonce_by_sender.insert(sender, nonce).is_some() {
        return invalid_payload(line);
    }
    Ok(())
}
