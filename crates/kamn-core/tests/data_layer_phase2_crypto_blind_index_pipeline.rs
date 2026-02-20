use std::collections::BTreeMap;

use kamn_core::{
    data_layer_phase2_build_operational_artifact, CanonicalMessageEnvelope,
    DataLayerPhase2OperationalPipelineError, DataLayerPhase2OperationalPipelineRequest,
    DataLayerPhase2RecipientEncryptionBinding, EnvelopeEncryption, EnvelopeHeader,
    EnvelopeMetadata, EnvelopeProof, CANONICAL_ENCRYPTION_ALGORITHM,
    CANONICAL_MESSAGE_ENVELOPE_TYPE, CANONICAL_PROOF_PURPOSE,
};

fn valid_envelope(message_id: &str) -> CanonicalMessageEnvelope {
    let mut body = BTreeMap::new();
    body.insert("content".to_owned(), "phase2 payload".to_owned());
    body.insert("topic".to_owned(), "alpha".to_owned());

    CanonicalMessageEnvelope {
        envelope: EnvelopeMetadata {
            id: message_id.to_owned(),
            type_name: CANONICAL_MESSAGE_ENVELOPE_TYPE.to_owned(),
            from: "kamn:did:agent:sender-1".to_owned(),
            to: vec![
                "kamn:did:agent:recipient-a".to_owned(),
                "kamn:did:agent:recipient-b".to_owned(),
            ],
            created: "2026-02-20T00:00:00Z".to_owned(),
            expires: "2026-02-20T01:00:00Z".to_owned(),
            thread_id: Some("thread-phase2".to_owned()),
            parent_id: None,
            nonce: 77,
        },
        header: EnvelopeHeader {
            message_type: "Request".to_owned(),
            priority: "normal".to_owned(),
            content_type: "application/json".to_owned(),
            encryption: EnvelopeEncryption {
                algorithm: CANONICAL_ENCRYPTION_ALGORITHM.to_owned(),
                recipient_keys: vec![
                    "did:key:z6MkrecipientA#key-agreement-1".to_owned(),
                    "did:key:z6MkrecipientB#key-agreement-1".to_owned(),
                ],
            },
        },
        body,
        attachments: Vec::new(),
        proof: EnvelopeProof {
            type_name: "DataIntegrityProof".to_owned(),
            created: "2026-02-20T00:00:00Z".to_owned(),
            verification_method: "kamn:did:agent:sender-1#key-1".to_owned(),
            proof_purpose: CANONICAL_PROOF_PURPOSE.to_owned(),
            proof_value: "sig:phase2".to_owned(),
        },
    }
}

fn valid_request(
    message_id: &str,
    recipient_bindings: Vec<DataLayerPhase2RecipientEncryptionBinding>,
) -> DataLayerPhase2OperationalPipelineRequest {
    let mut blind_index_fields = BTreeMap::new();
    blind_index_fields.insert("channel_topic".to_owned(), "alpha".to_owned());
    blind_index_fields.insert("message_type".to_owned(), "request".to_owned());

    DataLayerPhase2OperationalPipelineRequest {
        envelope: valid_envelope(message_id),
        sender_key_ref: "did:key:z6Mksender#key-agreement-1".to_owned(),
        recipient_bindings,
        blind_index_key_material: "owner-phase2-key".to_owned(),
        blind_index_fields,
        compression_dict_id: Some(9),
        hash_chain_prev: "sha256:phase2-prev".to_owned(),
        nonce: 9,
    }
}

#[test]
fn spec_c01_pipeline_artifacts_are_deterministic_across_binding_orderings() {
    let first = data_layer_phase2_build_operational_artifact(valid_request(
        "msg-phase2-c01",
        vec![
            DataLayerPhase2RecipientEncryptionBinding {
                recipient_did: "kamn:did:agent:recipient-b".to_owned(),
                recipient_key_ref: "did:key:z6MkrecipientB#key-agreement-1".to_owned(),
            },
            DataLayerPhase2RecipientEncryptionBinding {
                recipient_did: "kamn:did:agent:recipient-a".to_owned(),
                recipient_key_ref: "did:key:z6MkrecipientA#key-agreement-1".to_owned(),
            },
        ],
    ))
    .expect("first artifact should be derivable");
    let second = data_layer_phase2_build_operational_artifact(valid_request(
        "msg-phase2-c01",
        vec![
            DataLayerPhase2RecipientEncryptionBinding {
                recipient_did: "kamn:did:agent:recipient-a".to_owned(),
                recipient_key_ref: "did:key:z6MkrecipientA#key-agreement-1".to_owned(),
            },
            DataLayerPhase2RecipientEncryptionBinding {
                recipient_did: "kamn:did:agent:recipient-b".to_owned(),
                recipient_key_ref: "did:key:z6MkrecipientB#key-agreement-1".to_owned(),
            },
        ],
    ))
    .expect("second artifact should be derivable");

    assert_eq!(
        first.envelope_record.content_hash,
        second.envelope_record.content_hash
    );
    assert_eq!(
        first.envelope_record.envelope_ciphertext,
        second.envelope_record.envelope_ciphertext
    );
    assert_eq!(first.blind_indexes, second.blind_indexes);
}

#[test]
fn spec_c02_pipeline_rejects_invalid_sender_key_ref() {
    let mut request = valid_request(
        "msg-phase2-c02",
        vec![
            DataLayerPhase2RecipientEncryptionBinding {
                recipient_did: "kamn:did:agent:recipient-a".to_owned(),
                recipient_key_ref: "did:key:z6MkrecipientA#key-agreement-1".to_owned(),
            },
            DataLayerPhase2RecipientEncryptionBinding {
                recipient_did: "kamn:did:agent:recipient-b".to_owned(),
                recipient_key_ref: "did:key:z6MkrecipientB#key-agreement-1".to_owned(),
            },
        ],
    );
    request.sender_key_ref = "did:key:z6Mksender#wrong-fragment".to_owned();

    let error = data_layer_phase2_build_operational_artifact(request)
        .expect_err("invalid sender key ref should fail closed");
    assert!(matches!(
        error,
        DataLayerPhase2OperationalPipelineError::InvalidKeyRef { field, .. }
        if field == "sender_key_ref"
    ));
}

#[test]
fn spec_c02_pipeline_rejects_missing_recipient_bindings() {
    let request = valid_request("msg-phase2-c02-missing-bindings", Vec::new());
    let error = data_layer_phase2_build_operational_artifact(request)
        .expect_err("missing recipient bindings should fail closed");
    assert!(matches!(
        error,
        DataLayerPhase2OperationalPipelineError::MissingRecipientBindings { .. }
    ));
}
