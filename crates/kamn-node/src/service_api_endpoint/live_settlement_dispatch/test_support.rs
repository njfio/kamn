use super::models::{LiveSettlementEvidence, PreparedLiveSettlement};
use super::LiveSolanaSettlementConfig;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

static TEST_LIVE_SOLANA_SETTLEMENT_OVERRIDE: OnceLock<Mutex<bool>> = OnceLock::new();
static OBSERVED_SUBMITTED_INTENT: AtomicBool = AtomicBool::new(false);
static AMBIGUOUS_AFTER_SUBMIT: AtomicBool = AtomicBool::new(false);
static RECONCILE_CONFIRMED: AtomicBool = AtomicBool::new(false);
static SUBMISSION_COUNT: AtomicU64 = AtomicU64::new(0);
static EVIDENCE_MISMATCH: AtomicBool = AtomicBool::new(false);
static EXPIRED: AtomicBool = AtomicBool::new(false);

pub(crate) struct TestLiveSolanaSettlementOverrideGuard {
    previous: bool,
    previous_ambiguous: bool,
    previous_expired: bool,
}

pub(crate) fn set_test_live_solana_settlement_override(
    enabled: bool,
) -> TestLiveSolanaSettlementOverrideGuard {
    let mut guard = override_state()
        .lock()
        .expect("override lock should not poison");
    let previous = *guard;
    let previous_ambiguous = AMBIGUOUS_AFTER_SUBMIT.swap(false, Ordering::SeqCst);
    let previous_expired = EXPIRED.swap(false, Ordering::SeqCst);
    *guard = enabled;
    OBSERVED_SUBMITTED_INTENT.store(false, Ordering::SeqCst);
    RECONCILE_CONFIRMED.store(false, Ordering::SeqCst);
    SUBMISSION_COUNT.store(0, Ordering::SeqCst);
    EVIDENCE_MISMATCH.store(false, Ordering::SeqCst);
    TestLiveSolanaSettlementOverrideGuard {
        previous,
        previous_ambiguous,
        previous_expired,
    }
}

pub(crate) fn set_test_live_solana_settlement_reconcile_confirmed() {
    AMBIGUOUS_AFTER_SUBMIT.store(false, Ordering::SeqCst);
    RECONCILE_CONFIRMED.store(true, Ordering::SeqCst);
}

pub(crate) fn set_test_live_solana_settlement_evidence_mismatch() {
    EVIDENCE_MISMATCH.store(true, Ordering::SeqCst);
}

pub(crate) fn set_test_live_solana_settlement_expired() {
    AMBIGUOUS_AFTER_SUBMIT.store(false, Ordering::SeqCst);
    EXPIRED.store(true, Ordering::SeqCst);
}

pub(crate) fn test_live_solana_settlement_submission_count() -> u64 {
    SUBMISSION_COUNT.load(Ordering::SeqCst)
}

pub(crate) fn set_test_live_solana_settlement_ambiguous_after_submit(
) -> TestLiveSolanaSettlementOverrideGuard {
    let guard = set_test_live_solana_settlement_override(true);
    AMBIGUOUS_AFTER_SUBMIT.store(true, Ordering::SeqCst);
    guard
}

pub(crate) fn maybe_prepare_test_live_settlement(
    config: &LiveSolanaSettlementConfig,
    escrow_id: &str,
) -> Option<Result<PreparedLiveSettlement, String>> {
    if !*override_state()
        .lock()
        .expect("override lock should not poison")
    {
        return None;
    }
    Some(Ok(PreparedLiveSettlement {
        expected_signature: deterministic_signature(escrow_id),
        signed_transaction_digest: format!("sha256:test-{escrow_id}"),
        signed_transaction_json: format!("test-signed-transaction:{escrow_id}"),
        recipient_pubkey: config.recipient_pubkey.to_string(),
        amount_lamports: config.lamports,
        network: "solana:devnet".to_owned(),
    }))
}

pub(crate) fn maybe_submit_test_live_settlement(
    config: &LiveSolanaSettlementConfig,
    prepared: &PreparedLiveSettlement,
    escrow_id: &str,
    before_submit: &mut dyn FnMut() -> Result<(), String>,
) -> Option<Result<LiveSettlementEvidence, String>> {
    if !*override_state()
        .lock()
        .expect("override lock should not poison")
    {
        return None;
    }
    if RECONCILE_CONFIRMED.load(Ordering::SeqCst) {
        return Some(Ok(success_evidence(config, prepared)));
    }
    if EXPIRED.load(Ordering::SeqCst) {
        return Some(Err("SETTLEMENT_TRANSACTION_EXPIRED".to_owned()));
    }
    if let Err(error) = before_submit() {
        return Some(Err(error));
    }
    OBSERVED_SUBMITTED_INTENT.store(submitted_intent_exists(escrow_id), Ordering::SeqCst);
    SUBMISSION_COUNT.fetch_add(1, Ordering::SeqCst);
    if AMBIGUOUS_AFTER_SUBMIT.load(Ordering::SeqCst) {
        return Some(Err("SETTLEMENT_OUTCOME_AMBIGUOUS".to_owned()));
    }
    Some(Ok(success_evidence(config, prepared)))
}

fn success_evidence(
    config: &LiveSolanaSettlementConfig,
    prepared: &PreparedLiveSettlement,
) -> LiveSettlementEvidence {
    LiveSettlementEvidence {
        settlement_receipt_hash: prepared.expected_signature.clone(),
        settlement_tx_signature: prepared.expected_signature.clone(),
        settlement_network: prepared.network.clone(),
        settlement_commitment: config.commitment_label.clone(),
        recipient_pubkey: Some(prepared.recipient_pubkey.clone()),
        amount_lamports: Some(if EVIDENCE_MISMATCH.load(Ordering::SeqCst) {
            prepared.amount_lamports + 1
        } else {
            prepared.amount_lamports
        }),
    }
}

pub(crate) fn test_live_settlement_observed_submitted_intent() -> bool {
    OBSERVED_SUBMITTED_INTENT.load(Ordering::SeqCst)
}

impl Drop for TestLiveSolanaSettlementOverrideGuard {
    fn drop(&mut self) {
        let mut guard = override_state()
            .lock()
            .expect("override lock should not poison");
        *guard = self.previous;
        AMBIGUOUS_AFTER_SUBMIT.store(self.previous_ambiguous, Ordering::SeqCst);
        EXPIRED.store(self.previous_expired, Ordering::SeqCst);
    }
}

fn override_state() -> &'static Mutex<bool> {
    TEST_LIVE_SOLANA_SETTLEMENT_OVERRIDE.get_or_init(|| Mutex::new(false))
}

fn submitted_intent_exists(escrow_id: &str) -> bool {
    let Ok(path) = std::env::var("KAMN_SERVICE_API_STATE_FILE") else {
        return false;
    };
    let Ok(raw) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(state) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    state["settlement_intents"]
        .as_object()
        .and_then(|intents| intents.get(escrow_id))
        .is_some_and(|intent| {
            intent["state"] == "submitted" && intent["submission_attempt_count"] == 1
        })
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
