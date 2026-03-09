#![no_main]

use kamn_core::{
    AttachmentRef, CanonicalMessageEnvelope, EnvelopeEncryption, EnvelopeHeader, EnvelopeMetadata,
    EnvelopeProof,
};
use libfuzzer_sys::fuzz_target;
use std::collections::BTreeMap;

const DEFAULT_FROM_DID: &str = "kamn:did:agent:fuzz-sender-1";
const DEFAULT_TO_DID: &str = "kamn:did:agent:fuzz-recipient-1";
const DEFAULT_MESSAGE_TYPE: &str = "Request";
const DEFAULT_PRIORITY: &str = "normal";
const DEFAULT_CONTENT_TYPE: &str = "application/json";
const DEFAULT_ENCRYPTION_ALG: &str = "X25519-XChaCha20-Poly1305";
const DEFAULT_PROOF_PURPOSE: &str = "authentication";

fn bounded_utf8(data: &[u8], max_len: usize) -> String {
    let mut value = String::from_utf8_lossy(data).to_string();
    if value.len() > max_len {
        value.truncate(max_len);
    }
    value
}

fn bounded_field(parts: &[&str], index: usize, fallback: &str, max_len: usize) -> String {
    parts
        .get(index)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(max_len).collect())
        .unwrap_or_else(|| fallback.to_owned())
}

fn parse_csv(input: &str, fallback: &str, max_entries: usize, max_len: usize) -> Vec<String> {
    let mut values = input
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .take(max_entries)
        .map(|value| value.chars().take(max_len).collect::<String>())
        .collect::<Vec<_>>();
    if values.is_empty() {
        values.push(fallback.to_owned());
    }
    values
}

fn parse_body(input: &str) -> BTreeMap<String, String> {
    let mut body = BTreeMap::new();
    for pair in input.split(';').take(16) {
        let (key, value) = match pair.split_once('=') {
            Some((key, value)) => (key.trim(), value.trim()),
            None => continue,
        };
        if key.is_empty() || value.is_empty() {
            continue;
        }
        body.insert(
            key.chars().take(64).collect(),
            value.chars().take(256).collect(),
        );
    }
    if body.is_empty() {
        body.insert("msg".to_owned(), "fuzz".to_owned());
    }
    body
}

fn envelope_from_input(data: &[u8]) -> CanonicalMessageEnvelope {
    let raw = bounded_utf8(data, 4096);
    let parts = raw.split('|').take(16).collect::<Vec<_>>();
    CanonicalMessageEnvelope {
        envelope: build_metadata(&parts),
        header: build_header(&parts),
        body: build_body(&parts),
        attachments: vec![build_attachment(&parts)],
        proof: build_proof(&parts),
    }
}

fn build_metadata(parts: &[&str]) -> EnvelopeMetadata {
    EnvelopeMetadata {
        id: bounded_field(parts, 0, "fuzz-envelope-id", 64),
        type_name: bounded_field(parts, 1, "kamn:message:v1", 64),
        from: bounded_field(parts, 2, DEFAULT_FROM_DID, 96),
        to: parse_csv(parts.get(3).copied().unwrap_or_default(), DEFAULT_TO_DID, 6, 96),
        created: bounded_field(parts, 4, "2026-02-19T00:00:00Z", 48),
        expires: bounded_field(parts, 5, "2026-02-19T00:10:00Z", 48),
        thread_id: None,
        parent_id: None,
        nonce: 1,
    }
}

fn build_header(parts: &[&str]) -> EnvelopeHeader {
    EnvelopeHeader {
        message_type: bounded_field(parts, 6, DEFAULT_MESSAGE_TYPE, 32),
        priority: bounded_field(parts, 7, DEFAULT_PRIORITY, 16),
        content_type: bounded_field(parts, 8, DEFAULT_CONTENT_TYPE, 32),
        encryption: EnvelopeEncryption {
            algorithm: DEFAULT_ENCRYPTION_ALG.to_owned(),
            recipient_keys: parse_csv(
                parts.get(9).copied().unwrap_or_default(),
                "kamn:did:agent:fuzz-recipient-1#key-1",
                6,
                96,
            ),
        },
    }
}

fn build_body(parts: &[&str]) -> BTreeMap<String, String> {
    parse_body(parts.get(12).copied().unwrap_or_default())
}

fn build_attachment(parts: &[&str]) -> AttachmentRef {
    AttachmentRef {
        id: bounded_field(parts, 13, "attachment-1", 64),
        media_type: bounded_field(parts, 14, "text/plain", 64),
        uri: bounded_field(parts, 15, "ipfs://fuzz", 128),
    }
}

fn build_proof(parts: &[&str]) -> EnvelopeProof {
    EnvelopeProof {
        type_name: "Ed25519Signature2020".to_owned(),
        created: "2026-02-19T00:00:01Z".to_owned(),
        verification_method: bounded_field(
            parts,
            10,
            "kamn:did:agent:fuzz-sender-1#keys-1",
            128,
        ),
        proof_purpose: DEFAULT_PROOF_PURPOSE.to_owned(),
        proof_value: bounded_field(parts, 11, "proof-signature", 512),
    }
}

fuzz_target!(|data: &[u8]| {
    let envelope = envelope_from_input(data);
    let _ = envelope.validate();
    let _ = envelope.canonical_payload();
});
