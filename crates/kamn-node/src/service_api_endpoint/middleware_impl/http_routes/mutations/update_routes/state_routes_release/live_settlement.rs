use super::super::super::super::persistence_error as service_persistence_error;
use super::*;
mod bridge_authority;
mod errors;
mod prepared;
mod release_authority;
mod submission;
use bridge_authority::{
    consume_finalized_bridge_receipt, persist_bridge_intent, release_request,
    validate_bridge_settlement_authority,
};
use errors::{settlement_evidence_mismatch_error, settlement_intent_conflict_error};
use prepared::resolve_prepared;

type ReleaseResult = Result<Result<Option<ServiceApiEscrowStatusBody>, String>, Box<Response>>;
const BRIDGE_SETTLEMENT_AUTHORITY_MISMATCH: &str = "BRIDGE_SETTLEMENT_AUTHORITY_MISMATCH";

pub(super) async fn release(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
    escrow_id: &str,
    config: &LiveSolanaSettlementConfig,
) -> ReleaseResult {
    let mut store = state.message_store.lock().await;
    validate_release_eligibility(&mut store, context, escrow_id)?;
    let actor = super::super::super::super::task_actor(context)?;
    let request = release_request(context)?;
    if let Some(authority) = request.bridge_id.as_deref() {
        let authority = validate_bridge_settlement_authority(
            &mut store,
            config,
            actor.as_str(),
            escrow_id,
            authority,
        )?;
        if let Some(existing) = released_escrow(&mut store, escrow_id)? {
            return Ok(Ok(Some(existing)));
        }
        release_authority::persist(&mut store, context, escrow_id)?;
        persist_bridge_intent(
            &mut store,
            actor.as_str(),
            escrow_id,
            request.idempotency_key.as_str(),
            &authority,
        )?;
        let without_resubmission =
            consume_finalized_bridge_receipt(&mut store, escrow_id, &authority);
        return Ok(without_resubmission);
    }
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
    let request = release_request(context)?;
    persist_intent(
        store,
        actor.as_str(),
        escrow_id,
        request.idempotency_key.as_str(),
        prepared,
    )
}

fn released_escrow(
    store: &mut ServiceApiMessageStore,
    escrow_id: &str,
) -> Result<Option<ServiceApiEscrowStatusBody>, Box<Response>> {
    let existing = store
        .get_released_escrow_status(escrow_id)
        .map_err(persistence_error)?;
    Ok(existing)
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
    let result = live_settlement_dispatch::submit_or_reconcile_live_settlement(
        config,
        prepared,
        escrow_id,
        &mut persist,
    );
    submission::handle(store, escrow_id, result)
}

fn persistence_error(error: String) -> Box<Response> {
    Box::new(service_persistence_error(error.as_str()))
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
        .map_err(persistence_error)?;
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

fn settlement_metadata_from_evidence(
    evidence: LiveSettlementEvidence,
) -> ServiceApiSettlementMetadata {
    ServiceApiSettlementMetadata {
        settlement_receipt_hash: Some(evidence.settlement_receipt_hash),
        settlement_tx_signature: Some(evidence.settlement_tx_signature),
        settlement_network: Some(evidence.settlement_network),
        settlement_commitment: Some(evidence.settlement_commitment),
        ..ServiceApiSettlementMetadata::default()
    }
}
