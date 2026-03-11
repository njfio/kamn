use super::super::*;
use super::context::{RuntimeEvidenceContext, RuntimeEvidenceIdentities, RuntimeEvidenceM0ToM1};

pub(super) fn build_runtime_evidence_m0_to_m1(
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> Result<RuntimeEvidenceM0ToM1, String> {
    let m0_content_hash = build_runtime_evidence_m0_content_hash(context, identities)?;
    let m1_merkle_root = build_runtime_evidence_m1_merkle_root(context, &m0_content_hash)?;
    Ok(RuntimeEvidenceM0ToM1 {
        m0_content_hash,
        m1_merkle_root,
    })
}

fn build_runtime_evidence_m0_content_hash(
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> Result<String, String> {
    let mut m0_ledger = DataLayerM0AppendOnlyLedger::new();
    let m0_record = m0_ledger
        .append(build_runtime_evidence_m0_input(context, identities))
        .map_err(|error| format!("m0 runtime evidence failed: {error}"))?;
    m0_ledger
        .verify_hash_chain()
        .map_err(|error| format!("m0 hash-chain verification failed: {error}"))?;
    Ok(m0_record.content_hash)
}

fn build_runtime_evidence_m0_input(
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> DataLayerM0RecordInput {
    DataLayerM0RecordInput {
        envelope: build_runtime_evidence_envelope(context, identities),
        ciphertext: build_runtime_evidence_ciphertext(context),
        wrapped_keys: vec![DataLayerM0WrappedKey {
            did: identities.recipient_agent_did.clone(),
            wrapped_cek: format!("wrapped:{}", context.message_id),
        }],
        compression_codec: DATA_LAYER_M0_COMPRESSION_CODEC_ZSTD.to_owned(),
        compression_dict_id: Some(1),
        content_size_bytes: context.content_size_bytes,
        compressed_size_bytes: context.compressed_size_bytes,
    }
}

fn build_runtime_evidence_envelope(
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> CanonicalMessageEnvelope {
    CanonicalMessageEnvelope {
        envelope: build_runtime_evidence_metadata(context, identities),
        header: build_runtime_evidence_header(),
        body: build_runtime_evidence_body(context),
        attachments: Vec::new(),
        proof: build_runtime_evidence_proof(context, identities),
    }
}

fn build_runtime_evidence_metadata(
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> EnvelopeMetadata {
    EnvelopeMetadata {
        id: context.message_id.to_owned(),
        type_name: CANONICAL_MESSAGE_ENVELOPE_TYPE.to_owned(),
        from: identities.sender_agent_did.clone(),
        to: vec![identities.recipient_agent_did.clone()],
        created: "2026-02-24T00:00:00Z".to_owned(),
        expires: "2026-02-24T01:00:00Z".to_owned(),
        thread_id: Some(format!("thread:{}", context.message_id)),
        parent_id: None,
        nonce: (context.payload_tag % 1024).saturating_add(1),
    }
}

fn build_runtime_evidence_body(context: &RuntimeEvidenceContext<'_>) -> BTreeMap<String, String> {
    let mut envelope_body = BTreeMap::new();
    envelope_body.insert("payload".to_owned(), context.payload.to_owned());
    envelope_body.insert("runtime_message_id".to_owned(), context.message_id.to_owned());
    envelope_body
}

fn build_runtime_evidence_proof(
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> EnvelopeProof {
    EnvelopeProof {
        type_name: "DataIntegrityProof".to_owned(),
        created: "2026-02-24T00:00:00Z".to_owned(),
        verification_method: format!("{}#key-1", identities.sender_agent_did),
        proof_purpose: CANONICAL_PROOF_PURPOSE.to_owned(),
        proof_value: format!("sig:{}", context.message_id),
    }
}

fn build_runtime_evidence_header() -> EnvelopeHeader {
    EnvelopeHeader {
        message_type: "Request".to_owned(),
        priority: "normal".to_owned(),
        content_type: "application/json".to_owned(),
        encryption: EnvelopeEncryption {
            algorithm: CANONICAL_ENCRYPTION_ALGORITHM.to_owned(),
            recipient_keys: vec!["did:key:z6Mkrecipient#key-agreement-1".to_owned()],
        },
    }
}

fn build_runtime_evidence_ciphertext(context: &RuntimeEvidenceContext<'_>) -> DirectMessageCiphertext {
    DirectMessageCiphertext {
        key_agreement_algorithm: DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM.to_owned(),
        cipher_algorithm: DIRECT_MESSAGE_CIPHER_ALGORITHM.to_owned(),
        sender_key_ref: "did:key:z6Mksender#key-agreement-1".to_owned(),
        recipient_key_ref: "did:key:z6Mkrecipient#key-agreement-1".to_owned(),
        nonce: (context.payload_tag % 2048).saturating_add(1),
        ciphertext: format!("{:016x}", context.payload_tag),
        auth_tag: format!("{:032x}", context.payload_tag.saturating_add(1)),
    }
}

fn build_runtime_evidence_m1_merkle_root(
    context: &RuntimeEvidenceContext<'_>,
    m0_content_hash: &str,
) -> Result<String, String> {
    let m1_batch = DataLayerM1MerkleBatch::assemble(vec![
        DataLayerM1MerkleLeaf {
            message_id: context.message_id.to_owned(),
            leaf_index: 0,
            content_hash: m0_content_hash.to_owned(),
        },
        DataLayerM1MerkleLeaf {
            message_id: format!("{}:projection", context.message_id),
            leaf_index: 1,
            content_hash: format!("sha256:{:016x}", context.payload_tag),
        },
    ])
    .map_err(|error| format!("m1 merkle assembly failed: {error}"))?;
    let m1_proof = m1_batch
        .inclusion_proof(context.message_id)
        .map_err(|error| format!("m1 inclusion proof failed: {error}"))?;
    verify_data_layer_m1_inclusion_proof(&m1_proof)
        .map_err(|error| format!("m1 inclusion verification failed: {error}"))?;
    Ok(m1_batch.merkle_root)
}
