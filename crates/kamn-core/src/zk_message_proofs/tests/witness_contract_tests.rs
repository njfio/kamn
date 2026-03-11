use super::super::build_message_witness;
use crate::{
    AttachmentRef, CanonicalMessageEnvelope, EnvelopeEncryption, EnvelopeHeader, EnvelopeMetadata,
    EnvelopeProof,
};
use std::collections::BTreeMap;

#[test]
fn witness_builder_hides_selected_fields_and_builds_commitment() {
    let witness = build_message_witness(&valid_envelope(), &["task.description"])
        .expect("witness should build");
    assert!(witness.public_commitment.starts_with("fnv1a64:"));
    assert_eq!(witness.hidden_field_count, 1);
    assert_eq!(witness.revealed_fields, vec!["task.type".to_owned()]);
}

fn valid_envelope() -> CanonicalMessageEnvelope {
    CanonicalMessageEnvelope {
        envelope: valid_metadata(),
        header: valid_header(),
        body: valid_body(),
        attachments: valid_attachments(),
        proof: valid_proof(),
    }
}

fn valid_body() -> BTreeMap<String, String> {
    let mut body = BTreeMap::new();
    body.insert(
        "task.description".to_owned(),
        "Classify customer ticket".to_owned(),
    );
    body.insert("task.type".to_owned(), "support".to_owned());
    body
}

fn valid_metadata() -> EnvelopeMetadata {
    EnvelopeMetadata {
        id: "urn:uuid:420e8400-e29b-41d4-a716-446655440000".to_owned(),
        type_name: "kamn:message:v1".to_owned(),
        from: "kamn:did:agent:sender-1".to_owned(),
        to: vec!["kamn:did:agent:recipient-1".to_owned()],
        created: "2026-02-08T00:10:00.000Z".to_owned(),
        expires: "2026-02-08T00:40:00.000Z".to_owned(),
        thread_id: Some("urn:uuid:thread-9001".to_owned()),
        parent_id: None,
        nonce: 7,
    }
}

fn valid_header() -> EnvelopeHeader {
    EnvelopeHeader {
        message_type: "Request".to_owned(),
        priority: "Normal".to_owned(),
        content_type: "application/json".to_owned(),
        encryption: EnvelopeEncryption {
            algorithm: "X25519-XChaCha20-Poly1305".to_owned(),
            recipient_keys: vec!["kamn:did:agent:recipient-1#key-agreement-1".to_owned()],
        },
    }
}

fn valid_attachments() -> Vec<AttachmentRef> {
    vec![AttachmentRef {
        id: "attachment-1".to_owned(),
        media_type: "text/plain".to_owned(),
        uri: "ipfs://QmWitness".to_owned(),
    }]
}

fn valid_proof() -> EnvelopeProof {
    EnvelopeProof {
        type_name: "Ed25519Signature2020".to_owned(),
        created: "2026-02-08T00:10:00.000Z".to_owned(),
        verification_method: "kamn:did:agent:sender-1#keys-1".to_owned(),
        proof_purpose: "authentication".to_owned(),
        proof_value: "z58DAdFfa9SkqZ".to_owned(),
    }
}
