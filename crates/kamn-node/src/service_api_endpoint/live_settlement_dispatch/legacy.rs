use super::models::LiveSettlementEvidence;
use crate::service_api_endpoint::live_bridge_dispatch::{
    collect_live_solana_finalized_slot, LiveSolanaBridgeDispatchConfig,
};

pub(super) fn collect_slot_backed_live_settlement_evidence(
    config: &LiveSolanaBridgeDispatchConfig,
    escrow_id: &str,
) -> Result<LiveSettlementEvidence, String> {
    let finalized_slot = collect_live_solana_finalized_slot(config, escrow_id)?;
    let escrow_tag = crate::service_api_endpoint::deterministic_body_tag(
        format!("{escrow_id}:{finalized_slot}").as_bytes(),
    );
    let rpc_tag = crate::service_api_endpoint::deterministic_body_tag(config.rpc_url.as_bytes());
    let settlement_receipt_hash =
        format!("solana-devnet-settlement-{rpc_tag:016x}-{finalized_slot:016x}-{escrow_tag:016x}");
    Ok(LiveSettlementEvidence {
        settlement_receipt_hash,
        settlement_tx_signature: String::new(),
        settlement_network: String::new(),
        settlement_commitment: String::new(),
        recipient_pubkey: None,
        amount_lamports: None,
    })
}
