use super::support::*;
use super::*;

#[test]
fn functional_kolme_fork_submit_profile_uses_put_broadcast_and_maps_txhash_response() {
    let base_url = fork_txhash_server("ab12cd34", |request| {
        assert!(request.contains("Content-Type: application/json"));
        assert!(request.contains("X-Idempotency-Key: "));
    });

    let outcome = fork_provider(base_url.as_str(), "kolme-fork-local")
        .submit_runtime_commit(
            "operation_id=op-1\nstate_root=state-1\n",
            "kolme-runtime-commit:op-1:state-1:agent-1:1:payload-1",
        )
        .expect("submit should succeed");
    assert_pending_fork_receipt(outcome, "kolme-fork-local", "ab12cd34");
}

#[test]
fn regression_kolme_fork_submit_profile_requires_non_empty_provider_hint() {
    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let error = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        "http://127.0.0.1:3030",
        "",
        transport,
    )
    .expect_err("empty provider hint must fail validation");
    assert_eq!(error.to_string(), "invalid runtime commit request provider_hint: must not be empty");
}

#[test]
fn regression_issue_1914_kolme_fork_submit_profile_trims_provider_hint() {
    let base_url = fork_txhash_server("ab12cd34", |request| {
        assert!(request.contains("PUT /broadcast HTTP/1.1"));
    });

    let outcome = fork_provider(base_url.as_str(), "  kolme-fork-local  ")
        .submit_runtime_commit(
            "{\"message\":\"{\\\"pubkey\\\":\\\"pk1914\\\",\\\"nonce\\\":1,\\\"created\\\":\\\"2026-02-11T00:00:00Z\\\",\\\"messages\\\":[],\\\"max_height\\\":null}\",\"signature\":\"sig-1914\",\"recovery_id\":1}",
            "kolme-runtime-commit:provider-hint-1914",
        )
        .expect("submit should succeed");
    assert_pending_fork_receipt(outcome, "kolme-fork-local", "ab12cd34");
}
