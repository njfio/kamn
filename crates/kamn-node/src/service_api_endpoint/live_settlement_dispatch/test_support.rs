use super::models::LiveSettlementEvidence;
use super::LiveSolanaSettlementConfig;
use std::sync::{Mutex, OnceLock};

static TEST_LIVE_SOLANA_SETTLEMENT_OVERRIDE: OnceLock<Mutex<bool>> = OnceLock::new();

pub(crate) struct TestLiveSolanaSettlementOverrideGuard {
    previous: bool,
}

pub(crate) fn set_test_live_solana_settlement_override(
    enabled: bool,
) -> TestLiveSolanaSettlementOverrideGuard {
    let mut guard = override_state().lock().expect("override lock should not poison");
    let previous = *guard;
    *guard = enabled;
    TestLiveSolanaSettlementOverrideGuard { previous }
}

pub(crate) fn maybe_collect_test_live_settlement_evidence(
    config: &LiveSolanaSettlementConfig,
    escrow_id: &str,
) -> Option<Result<LiveSettlementEvidence, String>> {
    if !*override_state().lock().expect("override lock should not poison") {
        return None;
    }
    Some(Ok(LiveSettlementEvidence {
        settlement_receipt_hash: deterministic_signature(escrow_id),
        settlement_tx_signature: deterministic_signature(escrow_id),
        settlement_network: "solana:devnet".to_owned(),
        settlement_commitment: config.commitment_label.clone(),
    }))
}

impl Drop for TestLiveSolanaSettlementOverrideGuard {
    fn drop(&mut self) {
        let mut guard = override_state().lock().expect("override lock should not poison");
        *guard = self.previous;
    }
}

fn override_state() -> &'static Mutex<bool> {
    TEST_LIVE_SOLANA_SETTLEMENT_OVERRIDE.get_or_init(|| Mutex::new(false))
}

fn deterministic_signature(seed: &str) -> String {
    const BASE58: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut value = crate::service_api_endpoint::deterministic_body_tag(seed.as_bytes());
    let mut output = String::new();
    for _ in 0..64 {
        let index = (value % BASE58.len() as u64) as usize;
        output.push(BASE58[index] as char);
        value = value.rotate_left(5) ^ 0x9e37_79b9_7f4a_7c15;
    }
    output
}
