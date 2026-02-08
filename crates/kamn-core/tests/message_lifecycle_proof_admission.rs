use kamn_core::{
    build_message_witness, AttachmentRef, CanonicalMessageEnvelope, EnvelopeEncryption,
    EnvelopeHeader, EnvelopeMetadata, EnvelopeProof, MessageLifecycleStore,
    MessageProofAdmissionError, MessageStatus, ProcessorProofAdmissionEvaluator,
    ProcessorProofArtifact, ZkDesignError,
};
use std::collections::BTreeMap;

fn register_and_advance_delivered(store: &mut MessageLifecycleStore, message_id: &str) {
    store
        .register(
            message_id,
            "kamn:did:agent:sender-1",
            vec![
                "kamn:did:agent:recipient-1".to_owned(),
                "kamn:did:agent:recipient-2".to_owned(),
            ],
            "2026-02-09T00:10:00.000Z",
            "2026-02-09T00:40:00.000Z",
        )
        .expect("register should succeed");
    store
        .transition(message_id, MessageStatus::Signed)
        .expect("created->signed should succeed");
    store
        .transition(message_id, MessageStatus::Broadcast)
        .expect("signed->broadcast should succeed");
    store
        .transition(message_id, MessageStatus::Included)
        .expect("broadcast->included should succeed");
    store
        .transition(message_id, MessageStatus::Delivered)
        .expect("included->delivered should succeed");
}

fn envelope_for(message_id: &str) -> CanonicalMessageEnvelope {
    let mut body = BTreeMap::new();
    body.insert(
        "task.description".to_owned(),
        "Classify customer ticket".to_owned(),
    );
    body.insert("task.type".to_owned(), "support".to_owned());

    CanonicalMessageEnvelope {
        envelope: EnvelopeMetadata {
            id: message_id.to_owned(),
            type_name: "kamn:message:v1".to_owned(),
            from: "kamn:did:agent:sender-1".to_owned(),
            to: vec![
                "kamn:did:agent:recipient-1".to_owned(),
                "kamn:did:agent:recipient-2".to_owned(),
            ],
            created: "2026-02-09T00:10:00.000Z".to_owned(),
            expires: "2026-02-09T00:40:00.000Z".to_owned(),
            thread_id: Some("urn:uuid:thread-9001".to_owned()),
            parent_id: None,
            nonce: 9,
        },
        header: EnvelopeHeader {
            message_type: "Request".to_owned(),
            priority: "Normal".to_owned(),
            content_type: "application/json".to_owned(),
            encryption: EnvelopeEncryption {
                algorithm: "X25519-XChaCha20-Poly1305".to_owned(),
                recipient_keys: vec![
                    "kamn:did:agent:recipient-1#key-agreement-1".to_owned(),
                    "kamn:did:agent:recipient-2#key-agreement-1".to_owned(),
                ],
            },
        },
        body,
        attachments: vec![AttachmentRef {
            id: "attachment-1".to_owned(),
            media_type: "text/plain".to_owned(),
            uri: "ipfs://QmLifecycle".to_owned(),
        }],
        proof: EnvelopeProof {
            type_name: "Ed25519Signature2020".to_owned(),
            created: "2026-02-09T00:10:00.000Z".to_owned(),
            verification_method: "kamn:did:agent:sender-1#keys-1".to_owned(),
            proof_purpose: "authentication".to_owned(),
            proof_value: "z58DAdFfa9SkqZ".to_owned(),
        },
    }
}

#[test]
fn lifecycle_functional_valid_processor_proof_advances_to_validated() {
    let message_id = "urn:uuid:msg-proof-1";
    let mut store = MessageLifecycleStore::new();
    register_and_advance_delivered(&mut store, message_id);

    let witness = build_message_witness(&envelope_for(message_id), &["task.description"])
        .expect("witness should build");
    let artifact = ProcessorProofArtifact::new(
        "artifact-proof-1",
        message_id,
        &witness.public_commitment,
        "proof:ok:artifact-proof-1",
    )
    .expect("artifact should parse");
    let mut evaluator = ProcessorProofAdmissionEvaluator::new();

    store
        .validate_with_processor_proof(
            message_id,
            &witness.public_commitment,
            artifact,
            &mut evaluator,
        )
        .expect("valid proof should advance lifecycle");
    assert_eq!(
        store.status(message_id).expect("status should exist"),
        MessageStatus::Validated
    );
}

#[test]
fn lifecycle_regression_tampered_proof_does_not_advance_validation_state() {
    // Regression: #510
    let message_id = "urn:uuid:msg-proof-2";
    let mut store = MessageLifecycleStore::new();
    register_and_advance_delivered(&mut store, message_id);

    let witness = build_message_witness(&envelope_for(message_id), &["task.description"])
        .expect("witness should build");
    let artifact = ProcessorProofArtifact::new(
        "artifact-proof-2",
        message_id,
        "fnv1a64:tampered",
        "proof:ok:artifact-proof-2",
    )
    .expect("artifact should parse");
    let mut evaluator = ProcessorProofAdmissionEvaluator::new();

    assert_eq!(
        store.validate_with_processor_proof(
            message_id,
            &witness.public_commitment,
            artifact,
            &mut evaluator,
        ),
        Err(MessageProofAdmissionError::Proof(
            ZkDesignError::ProofArtifactCommitmentMismatch {
                expected: witness.public_commitment,
                found: "fnv1a64:tampered".to_owned(),
            }
        ))
    );
    assert_eq!(
        store
            .status(message_id)
            .expect("status should remain queryable"),
        MessageStatus::Delivered
    );
}

#[test]
fn lifecycle_integration_replayed_artifact_is_rejected_for_second_message() {
    let first_message_id = "urn:uuid:msg-proof-3";
    let second_message_id = "urn:uuid:msg-proof-4";
    let mut store = MessageLifecycleStore::new();
    register_and_advance_delivered(&mut store, first_message_id);
    register_and_advance_delivered(&mut store, second_message_id);

    let first_witness =
        build_message_witness(&envelope_for(first_message_id), &["task.description"])
            .expect("first witness should build");
    let second_witness =
        build_message_witness(&envelope_for(second_message_id), &["task.description"])
            .expect("second witness should build");
    let mut evaluator = ProcessorProofAdmissionEvaluator::new();

    store
        .validate_with_processor_proof(
            first_message_id,
            &first_witness.public_commitment,
            ProcessorProofArtifact::new(
                "artifact-replay",
                first_message_id,
                &first_witness.public_commitment,
                "proof:ok:artifact-replay",
            )
            .expect("artifact should parse"),
            &mut evaluator,
        )
        .expect("first message should validate");

    assert_eq!(
        store.validate_with_processor_proof(
            second_message_id,
            &second_witness.public_commitment,
            ProcessorProofArtifact::new(
                "artifact-replay",
                second_message_id,
                &second_witness.public_commitment,
                "proof:ok:artifact-replay",
            )
            .expect("artifact should parse"),
            &mut evaluator,
        ),
        Err(MessageProofAdmissionError::Proof(
            ZkDesignError::ProofArtifactReplay("artifact-replay".to_owned())
        ))
    );
    assert_eq!(
        store
            .status(second_message_id)
            .expect("status should remain queryable"),
        MessageStatus::Delivered
    );
}
