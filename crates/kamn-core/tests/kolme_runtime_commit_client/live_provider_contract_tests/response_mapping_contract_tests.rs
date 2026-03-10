use super::*;

#[test]
fn functional_live_provider_maps_submitted_json_response_to_provider_outcome() {
    let (outcome, calls, request) = submit_default_response(
        r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:runtime:55","finality":"final"}"#,
        "op-live-provider-001",
        "kamn:did:agent:live-provider-1",
        55,
        "payload:live-provider",
    );
    assert_eq!(
        outcome,
        submitted_receipt("kolme-commit:runtime:55", KolmeCommitReceiptFinality::Final)
    );
    assert_request_transport_call(
        &calls,
        &request,
        "http://127.0.0.1:3030",
        "/broadcast/runtime-commit",
    );
}

#[test]
fn regression_issue_1920_live_provider_trims_endpoint_inputs() {
    // Regression: #1920
    let (outcome, calls) = submit_trimmed_endpoint_response(
        r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:runtime:1920","finality":"pending"}"#,
        "op-live-provider-1920",
        "kamn:did:agent:live-provider-1920",
        61,
        "payload:live-provider-1920",
    );
    assert_eq!(
        outcome,
        submitted_receipt(
            "kolme-commit:runtime:1920",
            KolmeCommitReceiptFinality::Pending
        )
    );
    assert_single_transport_call(&calls, "http://127.0.0.1:3030", "/broadcast/runtime-commit");
}

#[test]
fn unit_kolme_fork_live_provider_maps_txhash_only_response_using_provider_hint() {
    let (outcome, calls) = submit_broadcast_profile_response(
        r#"{"txhash":"ab12cd34"}"#,
        "op-live-provider-1502-a",
        "kamn:did:agent:live-provider-1502-a",
        59,
    );
    assert_eq!(
        outcome,
        submitted_receipt("kolme-commit:ab12cd34", KolmeCommitReceiptFinality::Pending)
    );
    assert_single_transport_call(&calls, "http://127.0.0.1:3030", "/broadcast");
}

fn submit_default_response(
    response: &str,
    operation_id: &str,
    actor_did: &str,
    nonce: u64,
    payload_hash: &str,
) -> (
    KolmeRuntimeCommitProviderOutcome,
    TransportCalls,
    KolmeRuntimeCommitRequest,
) {
    let (transport, calls) = RecordingTransport::with_result(Ok(response.to_owned()));
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        "http://127.0.0.1:3030",
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");
    let request = build_request(operation_id, actor_did, nonce, payload_hash);
    let outcome = provider
        .submit_runtime_commit(&request.to_wire_payload(), request.idempotency_key())
        .expect("provider should return a parsed outcome");
    (outcome, calls, request)
}

fn submit_trimmed_endpoint_response(
    response: &str,
    operation_id: &str,
    actor_did: &str,
    nonce: u64,
    payload_hash: &str,
) -> (KolmeRuntimeCommitProviderOutcome, TransportCalls) {
    let (transport, calls) = RecordingTransport::with_result(Ok(response.to_owned()));
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        "  http://127.0.0.1:3030  ",
        "  /broadcast/runtime-commit  ",
        transport,
    )
    .expect("provider should build");
    let request = build_request(operation_id, actor_did, nonce, payload_hash);
    let outcome = provider
        .submit_runtime_commit(&request.to_wire_payload(), request.idempotency_key())
        .expect("provider should return a parsed outcome");
    (outcome, calls)
}

fn submit_broadcast_profile_response(
    response: &str,
    operation_id: &str,
    actor_did: &str,
    nonce: u64,
) -> (KolmeRuntimeCommitProviderOutcome, TransportCalls) {
    let (transport, calls) = RecordingTransport::with_result(Ok(response.to_owned()));
    let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        "http://127.0.0.1:3030",
        "kolme-fork-local",
        transport,
    )
    .expect("provider should build");
    let request = build_request(operation_id, actor_did, nonce, "payload:live-provider");
    let outcome = provider
        .submit_runtime_commit(&request.to_wire_payload(), request.idempotency_key())
        .expect("provider should map txhash-only response");
    (outcome, calls)
}

fn build_request(
    operation_id: &str,
    actor_did: &str,
    nonce: u64,
    payload_hash: &str,
) -> KolmeRuntimeCommitRequest {
    KolmeRuntimeCommitRequest::deterministic(
        operation_id,
        "state:live",
        actor_did,
        nonce,
        payload_hash,
    )
    .expect("request should build")
}

fn submitted_receipt(
    commit_id: &str,
    finality: KolmeCommitReceiptFinality,
) -> KolmeRuntimeCommitProviderOutcome {
    KolmeRuntimeCommitProviderOutcome::Submitted(KolmeRuntimeCommitProviderReceipt {
        provider: "kolme-fork-local".to_owned(),
        commit_id: commit_id.to_owned(),
        finality,
    })
}

fn assert_request_transport_call(
    calls: &TransportCalls,
    request: &KolmeRuntimeCommitRequest,
    base_url: &str,
    submit_path: &str,
) {
    let calls = calls.borrow();
    assert_eq!(calls.len(), 1, "provider transport should be called once");
    assert_eq!(calls[0].0, base_url);
    assert_eq!(calls[0].1, submit_path);
    assert_eq!(calls[0].2, request.to_wire_payload());
    assert_eq!(calls[0].3, request.idempotency_key());
}

fn assert_single_transport_call(calls: &TransportCalls, base_url: &str, submit_path: &str) {
    let calls = calls.borrow();
    assert_eq!(calls.len(), 1, "provider transport should be called once");
    assert_eq!(calls[0].0, base_url);
    assert_eq!(calls[0].1, submit_path);
}
