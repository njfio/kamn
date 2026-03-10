use super::*;

#[test]
fn regression_live_provider_fails_closed_for_malformed_response_shape() {
    // Regression: #1411
    let result = submit_result(
        r#"{"status":"submitted","provider":"kolme-fork-local","finality":"final"}"#,
        "op-live-provider-002",
        "kamn:did:agent:live-provider-2",
        56,
    );
    assert_malformed_response(
        result,
        "provider must fail closed for malformed backend responses",
    );
}

#[test]
fn regression_live_provider_rejects_statusless_response_without_txhash() {
    // Regression: #1502
    let result = submit_result(
        r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:runtime:missing-status"}"#,
        "op-live-provider-1502-b",
        "kamn:did:agent:live-provider-1502-b",
        60,
    );
    assert_malformed_response(
        result,
        "provider must fail closed when neither status nor txhash is present",
    );
}

fn submit_result(
    response: &str,
    operation_id: &str,
    actor_did: &str,
    nonce: u64,
) -> Result<KolmeRuntimeCommitProviderOutcome, KolmeRuntimeCommitProviderError> {
    let (transport, _calls) = RecordingTransport::with_result(Ok(response.to_owned()));
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        "http://127.0.0.1:3030",
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");
    let request = build_request(operation_id, actor_did, nonce, "payload:live-provider");
    provider.submit_runtime_commit(&request.to_wire_payload(), request.idempotency_key())
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

fn assert_malformed_response(
    result: Result<KolmeRuntimeCommitProviderOutcome, KolmeRuntimeCommitProviderError>,
    message: &str,
) {
    assert!(
        matches!(
            result,
            Err(KolmeRuntimeCommitProviderError::MalformedResponse { .. })
        ),
        "{message}"
    );
}
