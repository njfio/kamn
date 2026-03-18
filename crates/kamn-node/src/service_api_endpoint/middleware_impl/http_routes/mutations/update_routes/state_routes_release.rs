use super::*;
use crate::service_api_endpoint::live_settlement_dispatch::{
    LiveSettlementEvidence, LiveSolanaSettlementConfig,
};

pub(super) async fn resolve_release_escrow_result(
    state: &Arc<ServiceApiRuntimeState>,
    escrow_id: &str,
) -> Result<Result<Option<ServiceApiEscrowStatusBody>, String>, String> {
    if let Some(config) = state.live_solana_settlement.as_ref() {
        return release_escrow_with_live_solana_settlement(state, escrow_id, config).await;
    }
    let Some(config) = state.live_solana_bridge_dispatch.as_ref() else {
        return Ok(state.message_store.lock().await.release_escrow(escrow_id));
    };
    release_escrow_with_slot_backed_settlement(state, escrow_id, config).await
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
    escrow_id: &str,
    config: &LiveSolanaSettlementConfig,
) -> Result<Result<Option<ServiceApiEscrowStatusBody>, String>, String> {
    let mut store = state.message_store.lock().await;
    let existing = store.get_escrow_status(escrow_id)?;
    if existing.as_ref().is_some_and(|payload| payload.state == "released") {
        return Ok(Ok(existing));
    }
    let evidence =
        crate::service_api_endpoint::live_settlement_dispatch::collect_live_settlement_evidence(
            config, escrow_id,
        )?;
    Ok(store.release_escrow_with_settlement_metadata(
        escrow_id,
        &settlement_metadata_from_evidence(evidence),
    ))
}

async fn release_escrow_with_slot_backed_settlement(
    state: &Arc<ServiceApiRuntimeState>,
    escrow_id: &str,
    config: &LiveSolanaBridgeDispatchConfig,
) -> Result<Result<Option<ServiceApiEscrowStatusBody>, String>, String> {
    let evidence = crate::service_api_endpoint::live_settlement_dispatch::collect_slot_backed_live_settlement_evidence(
        config,
        escrow_id,
    )?;
    Ok(state
        .message_store
        .lock()
        .await
        .release_escrow_with_settlement_receipt_hash(
            escrow_id,
            evidence.settlement_receipt_hash.as_str(),
        ))
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
