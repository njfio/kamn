use super::*;
use crate::service_api_endpoint::live_settlement_dispatch::{
    LiveSettlementEvidence, LiveSolanaSettlementConfig, PreparedLiveSettlement,
};

pub(super) async fn resolve(
    state: &Arc<ServiceApiRuntimeState>,
    bridge_id: &str,
) -> Result<Result<Option<ServiceApiBridgeStatusBody>, String>, String> {
    let config = state.live_solana_settlement.as_ref().ok_or_else(|| {
        "BRIDGE_FINALITY_EVIDENCE_INVALID: live bridge requires transaction settlement config"
            .to_owned()
    })?;
    let mut store = state.message_store.lock().await;
    if let Some(existing) = finalized_bridge(&mut store, bridge_id)? {
        return Ok(Ok(Some(existing)));
    }
    let Some((prepared, subject)) =
        crate::service_api_endpoint::live_bridge_dispatch::resolve_prepared_bridge_transaction(
            &mut store, config, bridge_id,
        )?
    else {
        return Ok(Ok(None));
    };
    let evidence = submit_evidence(&mut store, config, &prepared, bridge_id, &subject)?;
    Ok(store.finalize_bridge_transaction(bridge_id, config, &evidence))
}

fn submit_evidence(
    store: &mut ServiceApiMessageStore,
    config: &LiveSolanaSettlementConfig,
    prepared: &PreparedLiveSettlement,
    bridge_id: &str,
    subject: &str,
) -> Result<LiveSettlementEvidence, String> {
    crate::service_api_endpoint::live_bridge_dispatch::submit_or_reconcile_bridge_transaction(
        store, config, prepared, bridge_id, subject,
    )
}

fn finalized_bridge(
    store: &mut ServiceApiMessageStore,
    bridge_id: &str,
) -> Result<Option<ServiceApiBridgeStatusBody>, String> {
    Ok(store
        .get_bridge(bridge_id)?
        .filter(|bridge| bridge.bridge_status == "finalized"))
}
