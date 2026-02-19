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
    let raw = std::str::from_utf8(data).unwrap_or_default();
    let parts = raw.split('|').take(16).collect::<Vec<_>>();

    let id = bounded_field(&parts, 0, "fuzz-envelope-id", 64);
    let type_name = bounded_field(&parts, 1, "kamn:message:v1", 64);
    let from = bounded_field(&parts, 2, DEFAULT_FROM_DID, 96);
    let to = parse_csv(
        parts.get(3).copied().unwrap_or_default(),
        DEFAULT_TO_DID,
        6,
        96,
    );
    let created = bounded_field(&parts, 4, "2026-02-19T00:00:00Z", 48);
    let expires = bounded_field(&parts, 5, "2026-02-19T00:10:00Z", 48);

    let message_type = bounded_field(&parts, 6, DEFAULT_MESSAGE_TYPE, 32);
    let priority = bounded_field(&parts, 7, DEFAULT_PRIORITY, 16);
    let content_type = bounded_field(&parts, 8, DEFAULT_CONTENT_TYPE, 32);
    let recipient_keys = parse_csv(
        parts.get(9).copied().unwrap_or_default(),
        "kamn:did:agent:fuzz-recipient-1#key-1",
        6,
        96,
    );

    let proof_verification_method = bounded_field(
        &parts,
        10,
        "kamn:did:agent:fuzz-sender-1#keys-1",
        128,
    );
    let proof_value = bounded_field(&parts, 11, "proof-signature", 512);

    let body = parse_body(parts.get(12).copied().unwrap_or_default());
    let attachment_id = bounded_field(&parts, 13, "attachment-1", 64);
    let attachment_media_type = bounded_field(&parts, 14, "text/plain", 64);
    let attachment_uri = bounded_field(&parts, 15, "ipfs://fuzz", 128);

    CanonicalMessageEnvelope {
        envelope: EnvelopeMetadata {
            id,
            type_name,
            from,
            to,
            created,
            expires,
            thread_id: None,
            parent_id: None,
            nonce: 1,
        },
        header: EnvelopeHeader {
            message_type,
            priority,
            content_type,
            encryption: EnvelopeEncryption {
                algorithm: DEFAULT_ENCRYPTION_ALG.to_owned(),
                recipient_keys,
            },
        },
        body,
        attachments: vec![AttachmentRef {
            id: attachment_id,
            media_type: attachment_media_type,
            uri: attachment_uri,
        }],
        proof: EnvelopeProof {
            type_name: "Ed25519Signature2020".to_owned(),
            created: "2026-02-19T00:00:01Z".to_owned(),
            verification_method: proof_verification_method,
            proof_purpose: DEFAULT_PROOF_PURPOSE.to_owned(),
            proof_value,
        },
    }
}

fuzz_target!(|data: &[u8]| {
    let envelope = envelope_from_input(data);
    let _ = envelope.validate();
    let _ = envelope.canonical_payload();
});
