use super::super::*;
use super::context::{RuntimeEvidenceContext, RuntimeEvidenceIdentities};

pub(super) fn build_runtime_evidence_m4(
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> Result<String, String> {
    let mut m4_escrow = DataLayerM4EscrowTransitionEngine::new();
    let m4_escrow_id = format!("escrow:{}", context.message_id);
    m4_escrow
        .create_escrow(build_runtime_evidence_m4_draft(context, identities, &m4_escrow_id))
        .map_err(|error| format!("m4 escrow draft failed: {error}"))?;
    apply_runtime_evidence_m4_transition(&mut m4_escrow, context, &m4_escrow_id)
}

fn build_runtime_evidence_m4_draft(
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
    escrow_id: &str,
) -> DataLayerM4EscrowDraftInput {
    DataLayerM4EscrowDraftInput {
        escrow_id: escrow_id.to_owned(),
        initiator_did: identities.sender_agent_did.clone(),
        counterparty_did: identities.recipient_agent_did.clone(),
        auditor_did: Some("kamn:did:auditor:service-api-runtime".to_owned()),
        auditor_threshold: Some(1),
        auditor_share_holders: vec!["kamn:did:holder:service-api-runtime".to_owned()],
        expires_at_epoch_seconds: Some(context.event_epoch_seconds.saturating_add(3_600)),
    }
}

fn apply_runtime_evidence_m4_transition(
    escrow: &mut DataLayerM4EscrowTransitionEngine,
    context: &RuntimeEvidenceContext<'_>,
    escrow_id: &str,
) -> Result<String, String> {
    let transition = escrow
        .apply_transition(
            escrow_id,
            DataLayerM4EscrowTransitionAction::Fund {
                funded_at_epoch_seconds: context.event_epoch_seconds.saturating_add(3),
            },
        )
        .map_err(|error| format!("m4 transition failed: {error}"))?;
    Ok(transition.reason_code.to_owned())
}

pub(super) fn build_runtime_evidence_m5(
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> Result<String, String> {
    let mut m5_registry =
        DataLayerM5EmbeddingRegistry::new(DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn);
    let m5_record = m5_registry
        .append(DataLayerM5EmbeddingRecordInput {
            embedding_id: format!("embed:{}", context.message_id),
            message_id: context.message_id.to_owned(),
            owner_did: identities.owner_did.to_owned(),
            agent_did: identities.sender_agent_did.clone(),
            retention_class: ContentRetentionClass::Standard,
            model_id: "text-embedding-3-large".to_owned(),
            vector_encrypted: vec![0xde, 0xad, 0xbe, 0xef],
            vector_plaintext: Some(vec![1.0, 0.0, 0.0]),
            created_at_epoch_seconds: context.event_epoch_seconds.saturating_add(4),
        })
        .map_err(|error| format!("m5 embedding append failed: {error}"))?;
    m5_registry
        .verify_owner_integrity(identities.owner_did)
        .map_err(|error| format!("m5 owner integrity failed: {error}"))?;
    Ok(m5_record.record_hash)
}
