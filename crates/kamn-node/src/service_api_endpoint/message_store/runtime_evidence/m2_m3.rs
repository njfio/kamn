use super::super::*;
use super::context::{RuntimeEvidenceContext, RuntimeEvidenceIdentities, RuntimeEvidenceM2ToM5};
use super::support::m2_authorization_reason_code;
use super::m4_m5::{build_runtime_evidence_m4, build_runtime_evidence_m5};

pub(super) fn build_runtime_evidence_m2_to_m5(
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> Result<RuntimeEvidenceM2ToM5, String> {
    let (m2_authorization_reason_code, m2_audit_record_hash, session_id) =
        build_runtime_evidence_m2(context, identities)?;
    let (m3_blind_index_token, m3_match_count) =
        build_runtime_evidence_m3(context, identities, session_id)?;
    let m4_transition_reason_code = build_runtime_evidence_m4(context, identities)?;
    let m5_record_hash = build_runtime_evidence_m5(context, identities)?;
    Ok(RuntimeEvidenceM2ToM5 {
        m2_authorization_reason_code,
        m2_audit_record_hash,
        m3_blind_index_token,
        m3_match_count,
        m4_transition_reason_code,
        m5_record_hash,
    })
}

fn build_runtime_evidence_m2(
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> Result<(String, String, String), String> {
    let auth_challenge = format!("nonce-{}", context.payload_tag);
    let m2_session_service =
        DataLayerM2DidSessionService::new(900).map_err(|error| format!("m2 init failed: {error}"))?;
    let m2_session_token = m2_session_service
        .authenticate(DataLayerM2DidAuthRequest {
            requester_did: identities.sender_agent_did.clone(),
            challenge: auth_challenge.clone(),
            credential: format!("sig:{}:{auth_challenge}", identities.sender_agent_did),
            issued_at_epoch_seconds: context.event_epoch_seconds,
            ttl_seconds: 300,
        })
        .map_err(|error| format!("m2 did authentication failed: {error}"))?;
    let authorization_reason_code = build_runtime_evidence_m2_authorization(context, identities)?;
    let audit_record_hash = build_runtime_evidence_m2_audit_hash(
        context,
        m2_session_token.requester_did,
        authorization_reason_code.as_str(),
    )?;
    Ok((authorization_reason_code, audit_record_hash, m2_session_token.token_id))
}

fn build_runtime_evidence_m2_authorization(
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> Result<String, String> {
    let scope = DataLayerM2MessageScope {
        message_id: context.message_id.to_owned(),
        sender_did: identities.sender_agent_did.clone(),
        recipient_did: identities.recipient_agent_did.clone(),
        owner_sender_did: identities.owner_did.to_owned(),
        owner_recipient_did: identities.owner_counterparty_did.to_owned(),
        escrow_id: None,
    };
    let decision = DataLayerM2AbacEngine::new()
        .authorize_message_visibility(
            identities.sender_agent_did.as_str(),
            DataLayerM2ActorRole::Agent,
            &scope,
        )
        .map_err(|error| format!("m2 authorization failed: {error}"))?;
    Ok(m2_authorization_reason_code(&decision))
}

fn build_runtime_evidence_m2_audit_hash(
    context: &RuntimeEvidenceContext<'_>,
    requester_did: String,
    reason_code: &str,
) -> Result<String, String> {
    let mut m2_audit_ledger = DataLayerM2AccessAuditLedger::new();
    let m2_audit_record = m2_audit_ledger
        .append(DataLayerM2AccessAuditInput {
            requester_did,
            action: "create_message".to_owned(),
            resource_id: context.message_id.to_owned(),
            reason_code: reason_code.to_owned(),
            event_epoch_seconds: context.event_epoch_seconds.saturating_add(1),
        })
        .map_err(|error| format!("m2 access audit append failed: {error}"))?;
    m2_audit_ledger
        .verify_hash_chain()
        .map_err(|error| format!("m2 access audit verification failed: {error}"))?;
    Ok(m2_audit_record.record_hash)
}

fn build_runtime_evidence_m3(
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
    session_id: String,
) -> Result<(String, usize), String> {
    let m3_blind_index_token = data_layer_m3_compute_blind_index(
        "service-api-runtime-owner-key",
        "message",
        context.payload,
    )
    .map_err(|error| format!("m3 blind-index compute failed: {error}"))?;
    let mut m3_catalog = DataLayerM3SearchCatalog::new();
    register_runtime_evidence_m3_record(
        &mut m3_catalog,
        context,
        identities,
        session_id,
        &m3_blind_index_token,
    )?;
    let matches = search_runtime_evidence_m3_matches(&m3_catalog, identities, &m3_blind_index_token)?;
    Ok((m3_blind_index_token, matches.len()))
}

fn register_runtime_evidence_m3_record(
    catalog: &mut DataLayerM3SearchCatalog,
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
    session_id: String,
    blind_index_token: &str,
) -> Result<(), String> {
    let mut blind_indexes = BTreeMap::new();
    blind_indexes.insert("message".to_owned(), blind_index_token.to_owned());
    catalog
        .register_record(DataLayerM3MessageMetadataRecord {
            message_id: context.message_id.to_owned(),
            owner_did: identities.owner_did.to_owned(),
            sender_did: identities.sender_agent_did.clone(),
            recipient_did: identities.recipient_agent_did.clone(),
            session_id: Some(session_id),
            escrow_id: None,
            message_type: "text".to_owned(),
            created_at_epoch_seconds: context.event_epoch_seconds.saturating_add(2),
            blind_indexes,
        })
        .map(|_| ())
        .map_err(|error| format!("m3 catalog registration failed: {error}"))
}

fn search_runtime_evidence_m3_matches(
    catalog: &DataLayerM3SearchCatalog,
    identities: &RuntimeEvidenceIdentities,
    blind_index_token: &str,
) -> Result<Vec<DataLayerM3MessageMetadataRecord>, String> {
    catalog
        .search_blind_index(DataLayerM3BlindIndexQuery {
            owner_did: identities.owner_did.to_owned(),
            field_name: "message".to_owned(),
            token: blind_index_token.to_owned(),
            mode: DataLayerM3BlindIndexSearchMode::ExactMatch,
            limit: Some(10),
        })
        .map_err(|error| format!("m3 search failed: {error}"))
}
