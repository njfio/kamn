use super::*;
use crate::service_api_endpoint::live_settlement_dispatch::{
    LiveSettlementEvidence, LiveSolanaSettlementConfig,
};

pub(super) async fn resolve_release_escrow_result(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
    escrow_id: &str,
) -> Result<Result<Option<ServiceApiEscrowStatusBody>, String>, Box<Response>> {
    if let Some(config) = state.live_solana_settlement.as_ref() {
        return release_escrow_with_live_solana_settlement(state, context, escrow_id, config).await;
    }
    let Some(config) = state.live_solana_bridge_dispatch.as_ref() else {
        let mut store = state.message_store.lock().await;
        validate_release_eligibility(&mut store, context, escrow_id)?;
        return Ok(store.release_escrow(escrow_id));
    };
    release_escrow_with_slot_backed_settlement(state, context, escrow_id, config).await
}

pub(super) fn live_settlement_evidence_error(error: &str) -> Response {
    super::payload::json_error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal",
        REASON_CODE_LIVE_SETTLEMENT_EVIDENCE_FAILED,
        format!("service api live settlement evidence failed: {error}").as_str(),
    )
}

async fn release_escrow_with_live_solana_settlement(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
    escrow_id: &str,
    config: &LiveSolanaSettlementConfig,
) -> Result<Result<Option<ServiceApiEscrowStatusBody>, String>, Box<Response>> {
    let mut store = state.message_store.lock().await;
    validate_release_eligibility(&mut store, context, escrow_id)?;
    let existing = store
        .get_escrow_status(escrow_id)
        .map_err(|error| Box::new(super::super::super::persistence_error(error.as_str())))?;
    if existing
        .as_ref()
        .is_some_and(|payload| payload.state == "released")
    {
        return Ok(Ok(existing));
    }
    let actor = super::super::super::task_actor(context)?;
    let key = release_idempotency_key(context)?;
    let prepared = resolve_prepared_settlement(&mut store, config, escrow_id)?;
    if let Err(error) =
        store.prepare_settlement_intent(actor.as_str(), escrow_id, key.as_str(), &prepared)
    {
        if error == "SETTLEMENT_INTENT_CONFLICT" {
            return Err(Box::new(settlement_intent_conflict_error()));
        }
        return Err(Box::new(live_settlement_evidence_error(error.as_str())));
    }
    let evidence = match crate::service_api_endpoint::live_settlement_dispatch::submit_or_reconcile_live_settlement(
        config, &prepared, escrow_id,
    ) {
        Ok(evidence) => evidence,
        Err(error) if error == "SETTLEMENT_OUTCOME_AMBIGUOUS" => {
            store.mark_settlement_outcome_ambiguous(escrow_id).map_err(|persist_error| {
                Box::new(super::super::super::persistence_error(persist_error.as_str()))
            })?;
            return Err(Box::new(settlement_outcome_ambiguous_error()));
        }
        Err(error) => return Err(Box::new(live_settlement_evidence_error(error.as_str()))),
    };
    Ok(store.finalize_settlement_intent(escrow_id, &settlement_metadata_from_evidence(evidence)))
}

fn settlement_outcome_ambiguous_error() -> Response {
    super::payload::json_error_response(
        StatusCode::SERVICE_UNAVAILABLE,
        "unavailable",
        "SETTLEMENT_OUTCOME_AMBIGUOUS",
        "settlement submission outcome is ambiguous and requires reconciliation",
    )
}

fn settlement_intent_conflict_error() -> Response {
    super::payload::json_error_response(
        StatusCode::CONFLICT,
        "conflict",
        "SETTLEMENT_INTENT_CONFLICT",
        "settlement idempotency key conflicts with the durable intent",
    )
}

fn resolve_prepared_settlement(
    store: &mut ServiceApiMessageStore,
    config: &LiveSolanaSettlementConfig,
    escrow_id: &str,
) -> Result<
    crate::service_api_endpoint::live_settlement_dispatch::PreparedLiveSettlement,
    Box<Response>,
> {
    let existing = store
        .get_settlement_intent(escrow_id)
        .map_err(|error| Box::new(super::super::super::persistence_error(error.as_str())))?;
    if let Some(intent) = existing {
        return Ok(prepared_from_intent(intent));
    }
    crate::service_api_endpoint::live_settlement_dispatch::prepare_live_settlement(
        config, escrow_id,
    )
    .map_err(|error| Box::new(live_settlement_evidence_error(error.as_str())))
}

fn prepared_from_intent(
    intent: ServiceApiSettlementIntentRecord,
) -> crate::service_api_endpoint::live_settlement_dispatch::PreparedLiveSettlement {
    crate::service_api_endpoint::live_settlement_dispatch::PreparedLiveSettlement {
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

fn invalid_release_key(message: &str) -> Response {
    super::payload::json_error_response(
        StatusCode::BAD_REQUEST,
        "bad_request",
        "ESCROW_AGREEMENT_INVALID",
        message,
    )
}

async fn release_escrow_with_slot_backed_settlement(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
    escrow_id: &str,
    config: &LiveSolanaBridgeDispatchConfig,
) -> Result<Result<Option<ServiceApiEscrowStatusBody>, String>, Box<Response>> {
    let mut store = state.message_store.lock().await;
    validate_release_eligibility(&mut store, context, escrow_id)?;
    let evidence = crate::service_api_endpoint::live_settlement_dispatch::collect_slot_backed_live_settlement_evidence(
        config,
        escrow_id,
    )
    .map_err(|error| Box::new(live_settlement_evidence_error(error.as_str())))?;
    Ok(store.release_escrow_with_settlement_receipt_hash(
        escrow_id,
        evidence.settlement_receipt_hash.as_str(),
    ))
}

fn revalidate_release(
    store: &mut ServiceApiMessageStore,
    context: &ServiceApiRequestContext,
) -> Result<(), Box<Response>> {
    super::super::super::revalidate_transaction_authorization(store, context)
}

fn validate_release_eligibility(
    store: &mut ServiceApiMessageStore,
    context: &ServiceApiRequestContext,
    escrow_id: &str,
) -> Result<(), Box<Response>> {
    revalidate_release(store, context)?;
    let actor = super::super::super::task_actor(context)?;
    store
        .validate_escrow_release_eligibility(actor.as_str(), escrow_id)
        .map_err(|error| Box::new(super::super::super::escrow_lifecycle_error_response(error)))
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
