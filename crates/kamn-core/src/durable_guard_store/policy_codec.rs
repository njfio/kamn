use crate::{PermissionRule, RetentionPolicy};
use std::collections::BTreeSet;

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

pub(crate) fn encode_permission_rule(rule: &PermissionRule) -> String {
    match rule {
        PermissionRule::All => "all".to_owned(),
        PermissionRule::Members => "members".to_owned(),
        PermissionRule::Admins => "admins".to_owned(),
        PermissionRule::Allowlist(values) => format_allowlist(values),
    }
}

pub(crate) fn decode_permission_rule(
    value: &str,
) -> Result<PermissionRule, super::DurableGuardSnapshotStoreError> {
    match value {
        "all" => Ok(PermissionRule::All),
        "members" => Ok(PermissionRule::Members),
        "admins" => Ok(PermissionRule::Admins),
        _ => parse_allowlist(value),
    }
}

pub(crate) fn encode_retention_policy(policy: &RetentionPolicy) -> String {
    match policy {
        RetentionPolicy::Forever => "forever".to_owned(),
        RetentionPolicy::MaxAgeSeconds(value) => format!("max_age:{value}"),
        RetentionPolicy::MaxMessageCount(value) => format!("max_count:{value}"),
    }
}

pub(crate) fn decode_retention_policy(
    value: &str,
) -> Result<RetentionPolicy, super::DurableGuardSnapshotStoreError> {
    if value == "forever" {
        return Ok(RetentionPolicy::Forever);
    }
    parse_numeric_retention(value)
}

pub(crate) fn encode_hex(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len() * 2);
    for byte in value.as_bytes() {
        encoded.push(char::from(HEX_CHARS[(byte >> 4) as usize]));
        encoded.push(char::from(HEX_CHARS[(byte & 0x0f) as usize]));
    }
    encoded
}

pub(crate) fn decode_hex(
    value: &str,
) -> Result<String, super::DurableGuardSnapshotStoreError> {
    if !value.len().is_multiple_of(2) {
        return invalid_payload(value);
    }
    let decoded = decode_hex_bytes(value)?;
    String::from_utf8(decoded).map_err(|_| invalid_payload_err(value))
}

fn format_allowlist(values: &BTreeSet<String>) -> String {
    let encoded_values = values
        .iter()
        .map(|value| encode_hex(value))
        .collect::<Vec<String>>()
        .join(",");
    format!("allowlist:{encoded_values}")
}

fn parse_allowlist(
    value: &str,
) -> Result<PermissionRule, super::DurableGuardSnapshotStoreError> {
    let encoded = value
        .strip_prefix("allowlist:")
        .ok_or_else(|| invalid_payload_err(value))?;
    let mut entries = BTreeSet::new();
    for token in encoded.split(',').filter(|token| !token.is_empty()) {
        entries.insert(decode_hex(token)?);
    }
    Ok(PermissionRule::Allowlist(entries))
}

fn parse_numeric_retention(
    value: &str,
) -> Result<RetentionPolicy, super::DurableGuardSnapshotStoreError> {
    if let Some(raw) = value.strip_prefix("max_age:") {
        let parsed = raw.parse::<u64>().map_err(|_| invalid_payload_err(value))?;
        return Ok(RetentionPolicy::MaxAgeSeconds(parsed));
    }
    if let Some(raw) = value.strip_prefix("max_count:") {
        let parsed = raw.parse::<usize>().map_err(|_| invalid_payload_err(value))?;
        return Ok(RetentionPolicy::MaxMessageCount(parsed));
    }
    invalid_payload(value)
}

fn decode_hex_bytes(
    value: &str,
) -> Result<Vec<u8>, super::DurableGuardSnapshotStoreError> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len() / 2);
    let mut index = 0;
    while index < bytes.len() {
        let high = decode_hex_nibble(bytes[index])?;
        let low = decode_hex_nibble(bytes[index + 1])?;
        decoded.push((high << 4) | low);
        index += 2;
    }
    Ok(decoded)
}

fn decode_hex_nibble(
    value: u8,
) -> Result<u8, super::DurableGuardSnapshotStoreError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(invalid_payload_err("invalid hex character")),
    }
}

fn invalid_payload<T>(value: &str) -> Result<T, super::DurableGuardSnapshotStoreError> {
    Err(invalid_payload_err(value))
}

fn invalid_payload_err(value: &str) -> super::DurableGuardSnapshotStoreError {
    super::DurableGuardSnapshotStoreError::InvalidPayload(value.to_owned())
}
