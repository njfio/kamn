use super::*;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub(crate) fn read_did_chain_adapter_file(
    path: &Path,
) -> Result<PersistedDidChainAdapterState, DidRegistryError> {
    if !path.exists() {
        return Ok((SubmissionReceiptIndex::new(), SubmissionRejectIndex::new()));
    }
    let payload = fs::read_to_string(path)
        .map_err(|error| DidRegistryError::PersistenceIo(error.to_string()))?;
    if payload.trim().is_empty() {
        return Ok((SubmissionReceiptIndex::new(), SubmissionRejectIndex::new()));
    }
    parse_did_chain_adapter_payload(&payload)
}

pub(crate) fn persist_did_chain_adapter_file(
    path: &Path,
    receipts_by_key: &SubmissionReceiptIndex,
    rejected_reasons_by_key: &SubmissionRejectIndex,
) -> Result<(), DidRegistryError> {
    let payload = serialize_did_chain_adapter_payload(receipts_by_key, rejected_reasons_by_key);
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .map_err(|error| DidRegistryError::PersistenceIo(error.to_string()))?;
    file.write_all(payload.as_bytes())
        .map_err(|error| DidRegistryError::PersistenceIo(error.to_string()))
}

fn serialize_did_chain_adapter_payload(
    receipts_by_key: &SubmissionReceiptIndex,
    rejected_reasons_by_key: &SubmissionRejectIndex,
) -> String {
    let mut lines = vec![format!(
        "schema|{}",
        super::super::models::DID_CHAIN_ADAPTER_SCHEMA_VERSION
    )];
    for (idempotency_key, reason) in rejected_reasons_by_key {
        lines.push(format!(
            "reject|{}|{}",
            encode_hex(idempotency_key.as_bytes()),
            encode_hex(reason.as_bytes())
        ));
    }
    for (idempotency_key, receipt) in receipts_by_key {
        lines.push(format!(
            "receipt|{}|{}|{}",
            encode_hex(idempotency_key.as_bytes()),
            encode_hex(receipt.provider.as_bytes()),
            encode_hex(receipt.transaction_id.as_bytes())
        ));
    }
    lines.join("\n")
}

fn parse_did_chain_adapter_payload(
    payload: &str,
) -> Result<PersistedDidChainAdapterState, DidRegistryError> {
    let mut lines = payload.lines().filter(|line| !line.trim().is_empty());
    let Some(schema_line) = lines.next() else {
        return Ok((SubmissionReceiptIndex::new(), SubmissionRejectIndex::new()));
    };
    let expected_schema = format!(
        "schema|{}",
        super::super::models::DID_CHAIN_ADAPTER_SCHEMA_VERSION
    );
    if schema_line != expected_schema {
        return Err(DidRegistryError::PersistenceInvalidPayload(
            schema_line.to_owned(),
        ));
    }
    let mut receipts_by_key = SubmissionReceiptIndex::new();
    let mut rejected_reasons_by_key = SubmissionRejectIndex::new();
    for line in lines {
        parse_payload_line(line, &mut receipts_by_key, &mut rejected_reasons_by_key)?;
    }
    Ok((receipts_by_key, rejected_reasons_by_key))
}

fn parse_payload_line(
    line: &str,
    receipts_by_key: &mut SubmissionReceiptIndex,
    rejected_reasons_by_key: &mut SubmissionRejectIndex,
) -> Result<(), DidRegistryError> {
    let mut parts = line.split('|');
    match parts.next() {
        Some("reject") => parse_reject_line(line, &mut parts, rejected_reasons_by_key),
        Some("receipt") => parse_receipt_line(line, &mut parts, receipts_by_key),
        _ => Err(DidRegistryError::PersistenceInvalidPayload(line.to_owned())),
    }
}

fn parse_reject_line(
    line: &str,
    parts: &mut std::str::Split<'_, char>,
    rejected_reasons_by_key: &mut SubmissionRejectIndex,
) -> Result<(), DidRegistryError> {
    let key = decode_required_string(parts.next(), line)?;
    let reason = decode_required_string(parts.next(), line)?;
    if parts.next().is_some() || rejected_reasons_by_key.insert(key, reason).is_some() {
        return Err(DidRegistryError::PersistenceInvalidPayload(line.to_owned()));
    }
    Ok(())
}

fn parse_receipt_line(
    line: &str,
    parts: &mut std::str::Split<'_, char>,
    receipts_by_key: &mut SubmissionReceiptIndex,
) -> Result<(), DidRegistryError> {
    let key = decode_required_string(parts.next(), line)?;
    let provider = decode_required_string(parts.next(), line)?;
    let transaction_id = decode_required_string(parts.next(), line)?;
    if parts.next().is_some() {
        return Err(DidRegistryError::PersistenceInvalidPayload(line.to_owned()));
    }
    let receipt = DidChainSubmissionReceipt {
        provider,
        transaction_id,
    };
    if receipts_by_key.insert(key, receipt).is_some() {
        return Err(DidRegistryError::PersistenceInvalidPayload(line.to_owned()));
    }
    Ok(())
}

fn decode_required_string(value: Option<&str>, line: &str) -> Result<String, DidRegistryError> {
    let value =
        value.ok_or_else(|| DidRegistryError::PersistenceInvalidPayload(line.to_owned()))?;
    let bytes = decode_hex(value)
        .ok_or_else(|| DidRegistryError::PersistenceInvalidPayload(line.to_owned()))?;
    String::from_utf8(bytes)
        .map_err(|_| DidRegistryError::PersistenceInvalidPayload(line.to_owned()))
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return None;
    }
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len() / 2);
    let mut index = 0usize;
    while index < bytes.len() {
        let high = decode_nibble(bytes[index])?;
        let low = decode_nibble(bytes[index + 1])?;
        decoded.push((high << 4) | low);
        index += 2;
    }
    Some(decoded)
}

fn decode_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}
