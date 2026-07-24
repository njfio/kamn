use super::*;

mod live;

pub(super) async fn forward_bridge(
    state: &Arc<ServiceApiRuntimeState>,
    bridge_id: &str,
) -> Response {
    let result = match resolve_forward_result(state, bridge_id).await {
        Ok(result) => result,
        Err(error) => return dispatch_error(error.as_str()),
    };
    match result {
        Ok(Some(payload)) => {
            state
                .websocket_events
                .publish_bridge_forwarded_event(&payload);
            contract_json(200, &payload)
        }
        Ok(None) => not_found(),
        Err(error) => persistence_error("service api bridge persistence failed", error),
    }
}

async fn resolve_forward_result(
    state: &Arc<ServiceApiRuntimeState>,
    bridge_id: &str,
) -> Result<Result<Option<ServiceApiBridgeStatusBody>, String>, String> {
    let Some(config) = state.live_solana_bridge_dispatch.as_ref() else {
        return Ok(state.message_store.lock().await.forward_bridge(bridge_id));
    };
    if state.live_solana_settlement.is_some() {
        return live::resolve(state, bridge_id).await;
    }
    let evidence =
        crate::service_api_endpoint::live_bridge_dispatch::collect_live_bridge_forward_evidence(
            config, bridge_id,
        )?;
    Ok(state
        .message_store
        .lock()
        .await
        .forward_bridge_with_evidence(
            bridge_id,
            evidence.target_message_id.as_str(),
            evidence.forward_tx_hash.as_str(),
        ))
}

fn dispatch_error(error: &str) -> Response {
    let (status, kind, code) = error_contract(error);
    super::super::payload::json_error_response(
        status,
        kind,
        code,
        format!("service api live bridge dispatch failed: {error}").as_str(),
    )
}

fn error_contract(error: &str) -> (StatusCode, &'static str, &'static str) {
    if error.contains("BRIDGE_RECONCILIATION_REQUIRED") {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "unavailable",
            "BRIDGE_RECONCILIATION_REQUIRED",
        );
    }
    if error.contains("BRIDGE_RECEIPT_REPLAY") {
        return (StatusCode::CONFLICT, "conflict", "BRIDGE_RECEIPT_REPLAY");
    }
    if error.contains("BRIDGE_FINALITY_EVIDENCE_INVALID") {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid",
            "BRIDGE_FINALITY_EVIDENCE_INVALID",
        );
    }
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal",
        REASON_CODE_LIVE_BRIDGE_DISPATCH_FAILED,
    )
}
