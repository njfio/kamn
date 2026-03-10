use super::*;
#[test]
fn functional_kolme_fork_submit_profile_uses_put_broadcast_and_maps_txhash_response() {
    let wire_payload = "operation_id=op-1\nstate_root=state-1\n";
    let idempotency_key = "kolme-runtime-commit:op-1:state-1:agent-1:1:payload-1";
    let base_url = spawn_single_request_server(
        "{\"txhash\":\"ab12cd34\"}".to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains("PUT /broadcast HTTP/1.1"));
            assert!(request.contains("Content-Type: application/json"));
            assert!(request.contains("X-Idempotency-Key: "));
        },
    );

    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        base_url.as_str(),
        "kolme-fork-local",
        transport,
    )
    .expect("provider should build");

    let outcome = provider
        .submit_runtime_commit(wire_payload, idempotency_key)
        .expect("submit should succeed");
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => {
            assert_eq!(receipt.provider, "kolme-fork-local");
            assert_eq!(receipt.commit_id, "kolme-commit:ab12cd34");
            assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Pending);
        }
        other => panic!("unexpected provider outcome: {other:?}"),
    }
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

    assert_eq!(
        error.to_string(),
        "invalid runtime commit request provider_hint: must not be empty"
    );
}

#[test]
fn regression_issue_1914_kolme_fork_submit_profile_trims_provider_hint() {
    // Regression: #1914
    let wire_payload = "{\"message\":\"{\\\"pubkey\\\":\\\"pk1914\\\",\\\"nonce\\\":1,\\\"created\\\":\\\"2026-02-11T00:00:00Z\\\",\\\"messages\\\":[],\\\"max_height\\\":null}\",\"signature\":\"sig-1914\",\"recovery_id\":1}";
    let idempotency_key = "kolme-runtime-commit:provider-hint-1914";

    let base_url = spawn_single_request_server(
        "{\"txhash\":\"ab12cd34\"}".to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains("PUT /broadcast HTTP/1.1"));
        },
    );

    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        base_url.as_str(),
        "  kolme-fork-local  ",
        transport,
    )
    .expect("provider should build");

    let outcome = provider
        .submit_runtime_commit(wire_payload, idempotency_key)
        .expect("submit should succeed");
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => {
            assert_eq!(receipt.provider, "kolme-fork-local");
            assert_eq!(receipt.commit_id, "kolme-commit:ab12cd34");
            assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Pending);
        }
        other => panic!("unexpected provider outcome: {other:?}"),
    }
}

