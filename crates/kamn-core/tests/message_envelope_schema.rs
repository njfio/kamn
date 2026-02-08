use kamn_core::{
    AttachmentRef, CanonicalMessageEnvelope, EnvelopeEncryption, EnvelopeHeader, EnvelopeMetadata,
    EnvelopeProof, MessageEnvelopeError,
};
use std::collections::BTreeMap;

fn body() -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    map.insert("task.type".to_owned(), "research".to_owned());
    map.insert(
        "task.description".to_owned(),
        "Analyze EV market trends in Southeast Asia".to_owned(),
    );
    map
}

fn valid_message() -> CanonicalMessageEnvelope {
    CanonicalMessageEnvelope {
        envelope: EnvelopeMetadata {
            id: "urn:uuid:550e8400-e29b-41d4-a716-446655440000".to_owned(),
            type_name: "kamn:message:v1".to_owned(),
            from: "kamn:did:agent:sender-1".to_owned(),
            to: vec!["kamn:did:agent:recipient-1".to_owned()],
            created: "2026-02-07T20:15:30.123Z".to_owned(),
            expires: "2026-02-07T20:45:30.123Z".to_owned(),
            thread_id: Some("urn:uuid:thread-123".to_owned()),
            parent_id: Some("urn:uuid:parent-456".to_owned()),
            nonce: 42,
        },
        header: EnvelopeHeader {
            message_type: "Request".to_owned(),
            priority: "Elevated".to_owned(),
            content_type: "application/json".to_owned(),
            encryption: EnvelopeEncryption {
                algorithm: "X25519-XChaCha20-Poly1305".to_owned(),
                recipient_keys: vec!["kamn:did:agent:recipient-1#key-agreement-1".to_owned()],
            },
        },
        body: body(),
        attachments: vec![AttachmentRef {
            id: "attachment-1".to_owned(),
            media_type: "application/pdf".to_owned(),
            uri: "ipfs://QmXoypiz".to_owned(),
        }],
        proof: EnvelopeProof {
            type_name: "Ed25519Signature2020".to_owned(),
            created: "2026-02-07T20:15:30.123Z".to_owned(),
            verification_method: "kamn:did:agent:sender-1#keys-1".to_owned(),
            proof_purpose: "authentication".to_owned(),
            proof_value: "z58DAdFfa9SkqZMVPxAQp".to_owned(),
        },
    }
}

#[test]
fn validates_canonical_message_schema_and_deterministic_payload() {
    let envelope = valid_message();
    assert!(envelope.validate().is_ok());
    assert_eq!(envelope.canonical_payload(), envelope.canonical_payload());
}

#[test]
fn rejects_unknown_envelope_type() {
    let mut envelope = valid_message();
    envelope.envelope.type_name = "kamn:message:v2".to_owned();

    assert_eq!(
        envelope.validate(),
        Err(MessageEnvelopeError::InvalidEnvelopeType(
            "kamn:message:v2".to_owned()
        ))
    );
}

#[test]
fn rejects_invalid_sender_did() {
    let mut envelope = valid_message();
    envelope.envelope.from = "did:example:agent-1".to_owned();

    assert!(matches!(
        envelope.validate(),
        Err(MessageEnvelopeError::InvalidSenderDid(_))
    ));
}

#[test]
fn rejects_invalid_expiry_window() {
    let mut envelope = valid_message();
    envelope.envelope.expires = envelope.envelope.created.clone();

    assert_eq!(
        envelope.validate(),
        Err(MessageEnvelopeError::InvalidExpiryWindow {
            created: "2026-02-07T20:15:30.123Z".to_owned(),
            expires: "2026-02-07T20:15:30.123Z".to_owned(),
        })
    );
}

#[test]
fn rejects_proof_verification_method_mismatch() {
    let mut envelope = valid_message();
    envelope.proof.verification_method = "kamn:did:agent:other#keys-1".to_owned();

    assert_eq!(
        envelope.validate(),
        Err(MessageEnvelopeError::ProofVerificationMethodMismatch {
            expected_prefix: "kamn:did:agent:sender-1#".to_owned(),
            actual: "kamn:did:agent:other#keys-1".to_owned(),
        })
    );
}

#[test]
fn rejects_nonce_zero_regression() {
    let mut envelope = valid_message();
    envelope.envelope.nonce = 0;

    // Regression: #113
    assert_eq!(
        envelope.validate(),
        Err(MessageEnvelopeError::InvalidNonce(0))
    );
}
