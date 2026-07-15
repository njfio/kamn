use super::finalize;

#[path = "agent_transaction_finalize_tests_support.rs"]
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

    let report = fixture.report();
    assert!(
        report.contains(r#"\"execution_surface\":\"live-service-persisted-receipt\""#),
        "canonical proof must use the persisted service receipt: {report}"
    );
    assert!(report.contains(r#"\"service_state_digest\":\"sha256:"#));
    assert!(report.contains(r#"\"settlement_intent_digest\":\"sha256:"#));
    assert!(report.contains(r#"\"fee_lamports\":5000"#));
    assert!(!report.contains("signed-transaction-secret"));

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

#[test]
fn proof_rejects_mismatched_persisted_settlement_intent() {
    let _guard = test_lock();
    let fixture = ProofRetryFixture::new();
    fixture.tamper_persisted_recipient();

    let error = finalize(&fixture.config, &fixture.paths)
        .expect_err("persisted recipient mismatch must fail closed");
    assert!(
        error.contains("persisted settlement recipient mismatch"),
        "{error}"
    );
}

fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
