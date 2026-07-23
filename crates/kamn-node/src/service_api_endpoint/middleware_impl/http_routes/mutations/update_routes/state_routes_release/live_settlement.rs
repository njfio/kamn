use super::*;

mod errors;
mod release_authority;
use errors::{
    invalid_release_key, settlement_evidence_mismatch_error, settlement_intent_conflict_error,
    settlement_outcome_ambiguous_error, settlement_transaction_expired_error,
};

type ReleaseResult = Result<Result<Option<ServiceApiEscrowStatusBody>, String>, Box<Response>>;

pub(super) async fn release(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
    escrow_id: &str,
    config: &LiveSolanaSettlementConfig,
) -> ReleaseResult {
    let mut store = state.message_store.lock().await;
    validate_release_eligibility(&mut store, context, escrow_id)?;
    if let Some(existing) = released_escrow(&mut store, escrow_id)? {
        return Ok(Ok(Some(existing)));
    }
    release_authority::persist(&mut store, context, escrow_id)?;
    let prepared = resolve_prepared(&mut store, config, escrow_id)?;
    persist_request_intent(&mut store, context, escrow_id, &prepared)?;
    let evidence = submit(&mut store, config, &prepared, escrow_id)?;
    validate_evidence(&mut store, config, &prepared, &evidence, escrow_id)?;
    Ok(store.finalize_settlement_intent(escrow_id, &settlement_metadata_from_evidence(evidence)))
}

fn persist_request_intent(
    store: &mut ServiceApiMessageStore,
    context: &ServiceApiRequestContext,
    escrow_id: &str,
    prepared: &PreparedLiveSettlement,
) -> Result<(), Box<Response>> {
    let actor = super::super::super::super::task_actor(context)?;
    let key = release_idempotency_key(context)?;
    persist_intent(store, actor.as_str(), escrow_id, key.as_str(), prepared)
}

fn released_escrow(
    store: &mut ServiceApiMessageStore,
    escrow_id: &str,
) -> Result<Option<ServiceApiEscrowStatusBody>, Box<Response>> {
    let existing = store.get_escrow_status(escrow_id).map_err(|error| {
        Box::new(super::super::super::super::persistence_error(
            error.as_str(),
        ))
    })?;
    Ok(existing.filter(|payload| payload.state == "released"))
}

fn persist_intent(
    store: &mut ServiceApiMessageStore,
    actor: &str,
    escrow_id: &str,
    key: &str,
    prepared: &PreparedLiveSettlement,
) -> Result<(), Box<Response>> {
    match store.prepare_settlement_intent(actor, escrow_id, key, prepared) {
        Ok(_) => Ok(()),
        Err(error) if error == "SETTLEMENT_INTENT_CONFLICT" => {
            Err(Box::new(settlement_intent_conflict_error()))
        }
        Err(error) => Err(Box::new(live_settlement_evidence_error(error.as_str()))),
    }
}

fn submit(
    store: &mut ServiceApiMessageStore,
    config: &LiveSolanaSettlementConfig,
    prepared: &PreparedLiveSettlement,
    escrow_id: &str,
) -> Result<LiveSettlementEvidence, Box<Response>> {
    let mut persist = || {
        store
            .mark_settlement_submitted(escrow_id)
            .map_err(|error| format!("SETTLEMENT_SUBMISSION_PERSISTENCE_FAILED: {error}"))
    };
    match live_settlement_dispatch::submit_or_reconcile_live_settlement(
        config,
        prepared,
        escrow_id,
        &mut persist,
    ) {
        Ok(evidence) => Ok(evidence),
        Err(error) if error.starts_with("SETTLEMENT_SUBMISSION_PERSISTENCE_FAILED") => {
            Err(persistence_error(error))
        }
        Err(error) if error.starts_with("SETTLEMENT_OUTCOME_AMBIGUOUS") => {
            persist_ambiguous(store, escrow_id)?;
            Err(Box::new(settlement_outcome_ambiguous_error()))
        }
        Err(error) if error == "SETTLEMENT_TRANSACTION_EXPIRED" => {
            persist_expired(store, escrow_id)?;
            Err(Box::new(settlement_transaction_expired_error()))
        }
        Err(error) => Err(Box::new(live_settlement_evidence_error(error.as_str()))),
    }
}

fn persist_ambiguous(
    store: &mut ServiceApiMessageStore,
    escrow_id: &str,
) -> Result<(), Box<Response>> {
    store
        .mark_settlement_outcome_ambiguous(escrow_id)
        .map_err(persistence_error)
}

fn persist_expired(
    store: &mut ServiceApiMessageStore,
    escrow_id: &str,
) -> Result<(), Box<Response>> {
    store
        .mark_settlement_expired(escrow_id)
        .map_err(persistence_error)
}

fn persistence_error(error: String) -> Box<Response> {
    Box::new(super::super::super::super::persistence_error(
        error.as_str(),
    ))
}

fn validate_evidence(
    store: &mut ServiceApiMessageStore,
    config: &LiveSolanaSettlementConfig,
    prepared: &PreparedLiveSettlement,
    evidence: &LiveSettlementEvidence,
    escrow_id: &str,
) -> Result<(), Box<Response>> {
    if evidence_matches(prepared, evidence, config) {
        return Ok(());
    }
    store
        .mark_settlement_failed(escrow_id, "SETTLEMENT_EVIDENCE_MISMATCH")
        .map_err(|error| {
            Box::new(super::super::super::super::persistence_error(
                error.as_str(),
            ))
        })?;
    Err(Box::new(settlement_evidence_mismatch_error()))
}

fn evidence_matches(
    prepared: &PreparedLiveSettlement,
    evidence: &LiveSettlementEvidence,
    config: &LiveSolanaSettlementConfig,
) -> bool {
    evidence.settlement_tx_signature == prepared.expected_signature
        && evidence.settlement_receipt_hash == prepared.expected_signature
        && evidence.settlement_network == prepared.network
        && evidence.settlement_commitment == config.commitment_label()
        && evidence.recipient_pubkey.as_deref() == Some(prepared.recipient_pubkey.as_str())
        && evidence.amount_lamports == Some(prepared.amount_lamports)
}

fn resolve_prepared(
    store: &mut ServiceApiMessageStore,
    config: &LiveSolanaSettlementConfig,
    escrow_id: &str,
) -> Result<PreparedLiveSettlement, Box<Response>> {
    let existing = store.get_settlement_intent(escrow_id).map_err(|error| {
        Box::new(super::super::super::super::persistence_error(
            error.as_str(),
        ))
    })?;
    if let Some(intent) = existing {
        return Ok(prepared_from_intent(intent));
    }
    live_settlement_dispatch::prepare_live_settlement(config, escrow_id)
        .map_err(|error| Box::new(live_settlement_evidence_error(error.as_str())))
}

fn prepared_from_intent(intent: ServiceApiSettlementIntentRecord) -> PreparedLiveSettlement {
    PreparedLiveSettlement {
        expected_signature: intent.expected_signature,
        signed_transaction_digest: intent.signed_transaction_digest,
        signed_transaction_json: intent.signed_transaction_json,
        recipient_pubkey: intent.recipient_pubkey,
        amount_lamports: intent.amount_lamports,
        network: intent.network,
    }
}

fn release_idempotency_key(context: &ServiceApiRequestContext) -> Result<String, Box<Response>> {
    let value: serde_json::Value = serde_json::from_str(context.parsed_request.body.as_str())
        .map_err(|error| Box::new(invalid_release_key(error.to_string().as_str())))?;
    value
        .get("idempotency_key")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| Box::new(invalid_release_key("release idempotency key is required")))
}

fn settlement_metadata_from_evidence(
    evidence: LiveSettlementEvidence,
) -> ServiceApiSettlementMetadata {
    ServiceApiSettlementMetadata {
        settlement_receipt_hash: Some(evidence.settlement_receipt_hash),
        settlement_tx_signature: Some(evidence.settlement_tx_signature),
        settlement_network: Some(evidence.settlement_network),
        settlement_commitment: Some(evidence.settlement_commitment),
    }
}
