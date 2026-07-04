use std::collections::BTreeMap;

use crate::{
    CanonicalMessageEnvelope, EnvelopeEncryption, EnvelopeHeader, EnvelopeMetadata, EnvelopeProof,
    CANONICAL_ENCRYPTION_ALGORITHM, CANONICAL_MESSAGE_ENVELOPE_TYPE, CANONICAL_PROOF_PURPOSE,
};

use super::NormalizedInboundMessage;

pub(super) fn build_canonical_envelope(
    bridge_agent_did: &str,
    normalized: NormalizedInboundMessage,
    recipient_keys: Vec<String>,
    expires: &str,
    nonce: u64,
) -> CanonicalMessageEnvelope {
    let body = build_body(&normalized);
    CanonicalMessageEnvelope {
        envelope: build_metadata(bridge_agent_did, &normalized, expires, nonce),
        header: build_header(recipient_keys),
        body,
        attachments: Vec::new(),
        proof: build_proof(bridge_agent_did, &normalized),
    }
}

fn build_body(normalized: &NormalizedInboundMessage) -> BTreeMap<String, String> {
    let mut body = BTreeMap::new();
    body.insert("message".to_owned(), normalized.body.clone());
    body.insert(
        "external_sender".to_owned(),
        normalized.sender_handle.clone(),
    );
    body.insert(
        "external_channel".to_owned(),
        normalized.source_channel.clone(),
    );
    body.insert("platform".to_owned(), normalized.platform.label());
    body
}

fn build_metadata(
    bridge_agent_did: &str,
    normalized: &NormalizedInboundMessage,
    expires: &str,
    nonce: u64,
) -> EnvelopeMetadata {
    EnvelopeMetadata {
        id: normalized.bridge_message_id.clone(),
        type_name: CANONICAL_MESSAGE_ENVELOPE_TYPE.to_owned(),
        from: bridge_agent_did.to_owned(),
        to: vec![normalized.target_agent_did.clone()],
        created: normalized.received_at.clone(),
        expires: expires.to_owned(),
        thread_id: None,
        parent_id: None,
        nonce,
    }
}

fn build_header(recipient_keys: Vec<String>) -> EnvelopeHeader {
    EnvelopeHeader {
        message_type: "Request".to_owned(),
        priority: "normal".to_owned(),
        content_type: "application/json".to_owned(),
        encryption: EnvelopeEncryption {
            algorithm: CANONICAL_ENCRYPTION_ALGORITHM.to_owned(),
            recipient_keys,
        },
    }
}

fn build_proof(bridge_agent_did: &str, normalized: &NormalizedInboundMessage) -> EnvelopeProof {
    EnvelopeProof {
        type_name: "Ed25519Signature2020".to_owned(),
        created: normalized.received_at.clone(),
        verification_method: format!("{bridge_agent_did}#bridge-key-1"),
        proof_purpose: CANONICAL_PROOF_PURPOSE.to_owned(),
        proof_value: format!("proof:{}", normalized.bridge_message_id),
    }
}
