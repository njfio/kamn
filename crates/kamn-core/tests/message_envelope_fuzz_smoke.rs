use kamn_core::{
    AttachmentRef, CanonicalMessageEnvelope, EnvelopeEncryption, EnvelopeHeader, EnvelopeMetadata,
    EnvelopeProof, MessageEnvelopeError,
};
use std::collections::BTreeMap;
use std::panic::catch_unwind;

fn base_body() -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    map.insert("task.type".to_owned(), "analysis".to_owned());
    map.insert(
        "task.description".to_owned(),
        "Generated fuzz-smoke payload".to_owned(),
    );
    map
}

fn valid_envelope() -> CanonicalMessageEnvelope {
    CanonicalMessageEnvelope {
        envelope: EnvelopeMetadata {
            id: "urn:uuid:fuzz-smoke".to_owned(),
            type_name: "kamn:message:v1".to_owned(),
            from: "kamn:did:agent:sender-1".to_owned(),
            to: vec!["kamn:did:agent:recipient-1".to_owned()],
            created: "2026-02-09T05:00:00.000Z".to_owned(),
            expires: "2026-02-09T05:10:00.000Z".to_owned(),
            thread_id: Some("urn:uuid:fuzz-thread".to_owned()),
            parent_id: None,
            nonce: 7,
        },
        header: EnvelopeHeader {
            message_type: "Request".to_owned(),
            priority: "Normal".to_owned(),
            content_type: "application/json".to_owned(),
            encryption: EnvelopeEncryption {
                algorithm: "X25519-XChaCha20-Poly1305".to_owned(),
                recipient_keys: vec!["kamn:did:agent:recipient-1#key-agreement-1".to_owned()],
            },
        },
        body: base_body(),
        attachments: vec![AttachmentRef {
            id: "attachment-1".to_owned(),
            media_type: "application/json".to_owned(),
            uri: "ipfs://bafybeifuzz".to_owned(),
        }],
        proof: EnvelopeProof {
            type_name: "Ed25519Signature2020".to_owned(),
            created: "2026-02-09T05:00:00.000Z".to_owned(),
            verification_method: "kamn:did:agent:sender-1#keys-1".to_owned(),
            proof_purpose: "authentication".to_owned(),
            proof_value: "z58DAdFfa9SkqZMVPxAQp".to_owned(),
        },
    }
}

fn mutation_slot(seed: u64, slots: usize) -> usize {
    // Stable tiny mixer; deterministic and allocation-free for fast smoke coverage.
    let mixed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (mixed % slots as u64) as usize
}

fn mutate_envelope_case(
    mut envelope: CanonicalMessageEnvelope,
    seed: u64,
) -> CanonicalMessageEnvelope {
    let variant = mutation_slot(seed, 12);
    let corpus_pick = mutation_slot(seed.rotate_left(17), 6);
    match variant {
        0 => {
            let ids = [
                "",
                " ",
                "urn:uuid:ok",
                "urn:uuid:edge",
                "x",
                "id-with-spaces ",
            ];
            envelope.envelope.id = ids[corpus_pick].to_owned();
        }
        1 => {
            let senders = [
                "did:example:sender",
                "kamn:did:agent:Sender-Upper",
                "kamn:did:agent:",
                "kamn:did:agent:sender-1",
                "kamn:did:agent:sender_2",
                "kamn:did:agent:sender 3",
            ];
            envelope.envelope.from = senders[corpus_pick].to_owned();
        }
        2 => {
            let recipients = [
                vec![],
                vec!["did:example:recipient".to_owned()],
                vec!["kamn:did:agent:recipient-1".to_owned()],
                vec!["kamn:did:agent:recipient upper".to_owned()],
                vec!["kamn:did:agent:recipient_2".to_owned()],
                vec!["kamn:did:agent:".to_owned()],
            ];
            envelope.envelope.to = recipients[corpus_pick].clone();
        }
        3 => {
            if corpus_pick.is_multiple_of(2) {
                envelope.envelope.expires = envelope.envelope.created.clone();
            } else {
                envelope.envelope.expires = "2026-01-01T00:00:00.000Z".to_owned();
            }
        }
        4 => {
            envelope.envelope.nonce = if corpus_pick.is_multiple_of(2) { 0 } else { 1 };
        }
        5 => {
            let message_types = [
                "Request",
                "Heartbeat",
                "UnknownType",
                "",
                " ",
                "PaymentOffer",
            ];
            envelope.header.message_type = message_types[corpus_pick].to_owned();
        }
        6 => {
            let algorithms = [
                "X25519-XChaCha20-Poly1305",
                "AES-GCM",
                "",
                " ",
                "curve25519",
                "X25519-XChaCha20-Poly1305",
            ];
            envelope.header.encryption.algorithm = algorithms[corpus_pick].to_owned();
        }
        7 => {
            let keys = [
                vec!["kamn:did:agent:recipient-1#key".to_owned()],
                vec![],
                vec!["".to_owned()],
                vec![" ".to_owned()],
                vec!["kamn:did:agent:recipient-2#key".to_owned()],
                vec!["kamn:did:agent:recipient-1#key".to_owned(), "".to_owned()],
            ];
            envelope.header.encryption.recipient_keys = keys[corpus_pick].clone();
        }
        8 => {
            if corpus_pick.is_multiple_of(2) {
                envelope.body.clear();
            } else {
                envelope
                    .body
                    .insert("".to_owned(), "non-empty-value".to_owned());
            }
        }
        9 => {
            if let Some(first_attachment) = envelope.attachments.first_mut() {
                if corpus_pick.is_multiple_of(2) {
                    first_attachment.id.clear();
                } else {
                    first_attachment.uri.clear();
                }
            }
        }
        10 => {
            let purposes = [
                "authentication",
                "assertionMethod",
                "",
                " ",
                "auth",
                "proof",
            ];
            envelope.proof.proof_purpose = purposes[corpus_pick].to_owned();
        }
        11 => {
            let methods = [
                "kamn:did:agent:sender-1#keys-1",
                "kamn:did:agent:other#keys-1",
                "",
                "sender#keys-1",
                "kamn:did:agent:sender-1",
                "did:example:sender#keys-1",
            ];
            envelope.proof.verification_method = methods[corpus_pick].to_owned();
        }
        _ => unreachable!("mutation slot is bounded"),
    }
    envelope
}

#[test]
fn fuzz_smoke_envelope_mutation_lane_is_panic_free_and_deterministic() {
    for seed in 0_u64..512 {
        let envelope = mutate_envelope_case(valid_envelope(), seed);

        let first = catch_unwind(|| envelope.validate());
        assert!(
            first.is_ok(),
            "envelope validate panicked in mutation seed {seed}"
        );

        let first = first.expect("panic-free result should unwrap");
        let second = envelope.validate();
        assert_eq!(
            first, second,
            "validation result should be deterministic for mutation seed {seed}"
        );
    }
}

#[test]
fn fuzz_smoke_envelope_error_corpus_covers_expected_rejection_classes() {
    let mut invalid_sender = valid_envelope();
    invalid_sender.envelope.from = "did:example:sender".to_owned();
    assert!(matches!(
        invalid_sender.validate(),
        Err(MessageEnvelopeError::InvalidSenderDid(_))
    ));

    let mut invalid_recipient = valid_envelope();
    invalid_recipient.envelope.to = vec!["did:example:recipient".to_owned()];
    assert!(matches!(
        invalid_recipient.validate(),
        Err(MessageEnvelopeError::InvalidRecipientDid(_))
    ));

    let mut invalid_message_type = valid_envelope();
    invalid_message_type.header.message_type = "UnknownType".to_owned();
    assert!(matches!(
        invalid_message_type.validate(),
        Err(MessageEnvelopeError::InvalidMessageType(_))
    ));

    let mut invalid_algorithm = valid_envelope();
    invalid_algorithm.header.encryption.algorithm = "AES-GCM".to_owned();
    assert!(matches!(
        invalid_algorithm.validate(),
        Err(MessageEnvelopeError::InvalidEncryptionAlgorithm(_))
    ));

    let mut empty_body = valid_envelope();
    empty_body.body.clear();
    assert_eq!(empty_body.validate(), Err(MessageEnvelopeError::EmptyBody));

    let mut invalid_proof = valid_envelope();
    invalid_proof.proof.proof_purpose = "assertionMethod".to_owned();
    assert!(matches!(
        invalid_proof.validate(),
        Err(MessageEnvelopeError::InvalidProofPurpose(_))
    ));
}
