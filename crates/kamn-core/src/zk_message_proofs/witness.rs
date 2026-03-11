use super::errors::ZkDesignError;
use crate::CanonicalMessageEnvelope;
use std::collections::BTreeSet;

/// Reduced witness summary emitted from a canonical message envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkMessageWitness {
    pub public_commitment: String,
    pub revealed_fields: Vec<String>,
    pub hidden_field_count: usize,
    pub payload_bytes: usize,
}

pub fn build_message_witness(
    envelope: &CanonicalMessageEnvelope,
    private_fields: &[&str],
) -> Result<ZkMessageWitness, ZkDesignError> {
    envelope.validate().map_err(ZkDesignError::EnvelopeError)?;
    let hidden = hidden_fields(envelope, private_fields)?;
    let canonical_payload = envelope.canonical_payload();
    let (redacted_body, revealed_fields) = redacted_body(envelope, &hidden);
    let hidden_list = hidden.iter().cloned().collect::<Vec<_>>().join(",");
    let commitment_input =
        format!("{canonical_payload}|redacted:{redacted_body}|hidden:{hidden_list}");
    Ok(ZkMessageWitness {
        public_commitment: format!("fnv1a64:{:016x}", fnv1a_64(commitment_input.as_bytes())),
        revealed_fields,
        hidden_field_count: hidden.len(),
        payload_bytes: canonical_payload.len(),
    })
}

fn hidden_fields(
    envelope: &CanonicalMessageEnvelope,
    private_fields: &[&str],
) -> Result<BTreeSet<String>, ZkDesignError> {
    let mut hidden = BTreeSet::new();
    for field in private_fields {
        validate_private_field_selector(field)?;
        validate_private_field_present(envelope, field)?;
        hidden.insert((*field).to_owned());
    }
    Ok(hidden)
}

fn validate_private_field_selector(field: &str) -> Result<(), ZkDesignError> {
    if field.trim().is_empty() {
        return Err(ZkDesignError::InvalidPrivateField(
            "private field names must not be empty".to_owned(),
        ));
    }
    if is_valid_private_field_selector(field) {
        return Ok(());
    }
    Err(ZkDesignError::InvalidPrivateField(format!(
        "private field selector `{field}` must contain only [A-Za-z0-9_.-] and no empty path segments"
    )))
}

fn validate_private_field_present(
    envelope: &CanonicalMessageEnvelope,
    field: &str,
) -> Result<(), ZkDesignError> {
    if envelope.body.contains_key(field) {
        return Ok(());
    }
    Err(ZkDesignError::MissingPrivateField(field.to_owned()))
}

fn redacted_body(
    envelope: &CanonicalMessageEnvelope,
    hidden: &BTreeSet<String>,
) -> (String, Vec<String>) {
    let mut redacted_body = String::new();
    let mut revealed_fields = Vec::new();
    for (key, value) in &envelope.body {
        redacted_body.push_str(key);
        redacted_body.push('=');
        if hidden.contains(key) {
            redacted_body.push_str("<hidden>");
        } else {
            redacted_body.push_str(value);
            revealed_fields.push(key.clone());
        }
        redacted_body.push(';');
    }
    (redacted_body, revealed_fields)
}

fn is_valid_private_field_selector(selector: &str) -> bool {
    let trimmed = selector.trim();
    if trimmed.is_empty() || trimmed.starts_with('.') || trimmed.ends_with('.') {
        return false;
    }
    if trimmed.contains("..") {
        return false;
    }
    trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
}

fn fnv1a_64(input: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET_BASIS;
    for byte in input {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}
