use super::*;
use crate::service_api_endpoint::live_settlement_dispatch::{
    LiveSettlementEvidence, LiveSolanaSettlementConfig, PreparedLiveSettlement,
};

mod live_settlement;

pub(super) async fn resolve_release_escrow_result(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
    escrow_id: &str,
) -> Result<Result<Option<ServiceApiEscrowStatusBody>, String>, Box<Response>> {
    if let Some(config) = state.live_solana_settlement.as_ref() {
        return live_settlement::release(state, context, escrow_id, config).await;
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
