use super::finalize;

#[path = "agent_transaction_finalize_test_support.rs"]
mod support;
use support::ProofRetryFixture;

#[test]
fn proof_retry_reuses_persisted_settlement_without_submission() {
    let _guard = test_lock();
    let fixture = ProofRetryFixture::new();
    fixture.block_latest_publication();

    let first =
        finalize(&fixture.config, &fixture.paths).expect_err("latest publication must fail");
    assert!(
        first.contains("failed to remove previous latest demo"),
        "unexpected proof failure: {first}"
    );
    fixture.unblock_latest_publication();
    finalize(&fixture.config, &fixture.paths).expect("proof retry should pass");

    let calls = fixture.solana_calls();
    assert_eq!(
        calls
            .lines()
            .filter(|line| line.starts_with("confirm "))
            .count(),
        2
    );
    assert!(!calls.contains("transfer"));
    assert!(!calls.contains("send"));
}

fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
