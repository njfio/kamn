use super::*;

#[test]
fn funded_rehearsal_payload_carries_canonical_task_agreement() {
    let run_dir = Path::new("/tmp/run-contract");
    let binding = LiveTaskBinding {
        artifact_path: "binding.json".to_owned(),
        digest: "a".repeat(64),
        task_id: "task-external".to_owned(),
        agent_a_pid: 1,
        agent_b_pid: 2,
        agent_c_pid: 3,
    };
    let creator = agent("http://127.0.0.1:1", "contract-creator").expect("creator");
    let provider = agent("http://127.0.0.1:1", "contract-provider").expect("provider");
    let agreement =
        SettlementAgreement::new(run_dir, Some(&binding), 1_000_000, &creator, &provider)
            .expect("agreement");
    let raw = agreement.fund_payload("task-settlement");
    for field in required_funding_fields() {
        assert!(
            raw.contains(format!("\"{field}\":").as_str()),
            "missing {field}"
        );
    }
    assert!(raw.contains(r#""network":"solana-devnet""#));
    assert!(raw.contains(r#""release_policy":"task-completed""#));
}

fn required_funding_fields() -> [&'static str; 9] {
    [
        "task_id",
        "transaction_id",
        "beneficiary_did",
        "amount_lamports",
        "network",
        "terms_digest",
        "release_authority_did",
        "release_policy",
        "idempotency_key",
    ]
}

#[test]
fn funded_rehearsal_paces_release_past_sender_window() {
    assert!(release_pacing_delay() > std::time::Duration::from_secs(5));
}

#[test]
fn funded_rehearsal_retries_only_recoverable_settlement_visibility() {
    assert_eq!(release_attempt_limit(), 15);
    assert!(should_retry_release("live settlement evidence failed"));
    assert!(should_retry_release("SETTLEMENT_OUTCOME_AMBIGUOUS"));
    assert!(!should_retry_release("ACTION_NOT_GRANTED"));
    assert!(!should_retry_release("SETTLEMENT_INTENT_CONFLICT"));
}
