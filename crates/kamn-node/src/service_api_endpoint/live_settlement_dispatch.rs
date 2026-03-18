use super::live_bridge_dispatch::{
    collect_live_solana_finalized_slot, LiveSolanaBridgeDispatchConfig,
};
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LiveSettlementEvidence {
    pub(super) settlement_receipt_hash: String,
}

pub(super) fn collect_live_settlement_evidence(
    config: &LiveSolanaBridgeDispatchConfig,
    escrow_id: &str,
) -> Result<LiveSettlementEvidence, String> {
    let finalized_slot = collect_live_solana_finalized_slot(config, escrow_id)?;
    Ok(build_live_settlement_evidence(
        escrow_id,
        finalized_slot,
        config.rpc_url.as_str(),
    ))
}

fn build_live_settlement_evidence(
    escrow_id: &str,
    finalized_slot: u64,
    rpc_url: &str,
) -> LiveSettlementEvidence {
    let escrow_tag = deterministic_body_tag(format!("{escrow_id}:{finalized_slot}").as_bytes());
    let rpc_tag = deterministic_body_tag(rpc_url.as_bytes());
    LiveSettlementEvidence {
        settlement_receipt_hash: format!(
            "solana-devnet-settlement-{rpc_tag:016x}-{finalized_slot:016x}-{escrow_tag:016x}"
        ),
    }
}
