mod config;
mod legacy;
mod models;
#[cfg(test)]
mod test_support;
mod transport;

pub(crate) use config::{resolve_live_solana_settlement_config, LiveSolanaSettlementConfig};
pub(crate) use models::{LiveSettlementEvidence, PreparedLiveSettlement};

pub(super) fn collect_slot_backed_live_settlement_evidence(
    config: &super::live_bridge_dispatch::LiveSolanaBridgeDispatchConfig,
    escrow_id: &str,
) -> Result<LiveSettlementEvidence, String> {
    legacy::collect_slot_backed_live_settlement_evidence(config, escrow_id)
}

pub(super) fn prepare_live_settlement(
    config: &LiveSolanaSettlementConfig,
    escrow_id: &str,
) -> Result<PreparedLiveSettlement, String> {
    transport::prepare_live_settlement(config, escrow_id)
}

pub(super) fn submit_or_reconcile_live_settlement(
    config: &LiveSolanaSettlementConfig,
    prepared: &PreparedLiveSettlement,
    escrow_id: &str,
) -> Result<LiveSettlementEvidence, String> {
    transport::submit_or_reconcile_live_settlement(config, prepared, escrow_id)
}

#[cfg(test)]
pub(crate) use test_support::{
    set_test_live_solana_settlement_ambiguous_after_submit,
    set_test_live_solana_settlement_evidence_mismatch, set_test_live_solana_settlement_expired,
    set_test_live_solana_settlement_override, set_test_live_solana_settlement_reconcile_confirmed,
    test_live_settlement_observed_submitted_intent, test_live_solana_settlement_submission_count,
};
