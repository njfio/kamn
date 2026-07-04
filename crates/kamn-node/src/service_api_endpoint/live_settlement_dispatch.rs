mod config;
mod legacy;
mod models;
#[cfg(test)]
mod test_support;
mod transport;

pub(crate) use config::{resolve_live_solana_settlement_config, LiveSolanaSettlementConfig};
pub(crate) use models::LiveSettlementEvidence;

pub(super) fn collect_slot_backed_live_settlement_evidence(
    config: &super::live_bridge_dispatch::LiveSolanaBridgeDispatchConfig,
    escrow_id: &str,
) -> Result<LiveSettlementEvidence, String> {
    legacy::collect_slot_backed_live_settlement_evidence(config, escrow_id)
}

pub(super) fn collect_live_settlement_evidence(
    config: &LiveSolanaSettlementConfig,
    escrow_id: &str,
) -> Result<LiveSettlementEvidence, String> {
    transport::collect_live_settlement_evidence(config, escrow_id)
}

#[cfg(test)]
pub(crate) use test_support::set_test_live_solana_settlement_override;
