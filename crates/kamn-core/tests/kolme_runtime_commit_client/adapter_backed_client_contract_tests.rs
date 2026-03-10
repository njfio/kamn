use super::support::*;

#[test]
fn unit_adapter_normalizes_wire_payload_and_idempotency_key_before_submit() {
    let request = build_request("op-sync-adapter-100", 9, "runtime-node-adapter-1");
    let expected_payload = request.to_wire_payload();
    let expected_key = request.idempotency_key().to_owned();
    let (mut client, calls) = client_with_receipt(
        "kolme-local",
        "kolme-local",
        "kolme-commit:adapter-100",
        KolmeCommitReceiptFinality::Final,
    );

    let outcome = client
        .submit_commit(&request)
        .expect("adapter submit should succeed");

    assert!(matches!(outcome, KolmeRuntimeCommitOutcome::Submitted(_)));
    assert_submitted_call(&calls, &expected_payload, &expected_key);
}

#[test]
fn functional_adapter_maps_transport_provider_and_finality_failures_to_typed_errors() {
    let request = build_request("op-sync-adapter-101", 4, "runtime-node-adapter-2");
    assert_timeout_failure(&request);
    assert_provider_mismatch_failure(&request);
    assert_non_final_failure(&request);
}

#[test]
fn integration_runtime_pipeline_accepts_adapter_backed_final_receipts() {
    let request = build_request("op-sync-adapter-103", 5, "runtime-node-adapter-3");
    let (mut client, _calls) = client_with_receipt(
        "kolme-local",
        "kolme-local",
        "kolme-commit:adapter-103",
        KolmeCommitReceiptFinality::Final,
    );
    let mut pipeline = RuntimeCommitPipeline::new();

    let record = pipeline
        .submit_with_client(&mut client, request)
        .expect("pipeline submit should succeed");

    assert_eq!(record.state, RuntimeCommitLifecycleState::Finalized);
    assert!(!record.needs_requeue);
}

#[test]
fn regression_adapter_path_keeps_receipt_provider_mismatch_fail_closed() {
    // Regression: #979
    let request = build_request("op-sync-adapter-104", 6, "runtime-node-adapter-4");
    let (mut client, _calls) = client_with_receipt(
        "kolme-local",
        "kolme-local",
        "kolme-commit:adapter-104",
        KolmeCommitReceiptFinality::Final,
    );
    let mut pipeline = RuntimeCommitPipeline::new();
    let record = pipeline
        .submit_with_client(&mut client, request)
        .expect("pipeline submit should succeed");

    assert_eq!(record.receipt_provider.as_deref(), Some("kolme-local"));
    assert_provider_mismatch_receipt_error(&mut pipeline);
}

fn build_request(operation_id: &str, nonce: u64, actor_suffix: &str) -> KolmeRuntimeCommitRequest {
    KolmeRuntimeCommitRequest::deterministic(
        operation_id,
        "state:adapter",
        &format!("kamn:did:agent:{actor_suffix}"),
        nonce,
        "payload:adapter",
    )
    .expect("request should build")
}

fn client_with_receipt(
    expected_provider: &str,
    observed_provider: &str,
    commit_id: &str,
    finality: KolmeCommitReceiptFinality,
) -> (
    AdapterBackedKolmeRuntimeCommitClient<RecordingProvider>,
    ProviderCalls,
) {
    let (provider, calls) = RecordingProvider::with_result(Ok(submitted_receipt(
        observed_provider,
        commit_id,
        finality,
    )));
    let client =
        AdapterBackedKolmeRuntimeCommitClient::new(expected_provider, provider).expect("client");
    (client, calls)
}

fn submitted_receipt(
    provider: &str,
    commit_id: &str,
    finality: KolmeCommitReceiptFinality,
) -> KolmeRuntimeCommitProviderOutcome {
    KolmeRuntimeCommitProviderOutcome::Submitted(KolmeRuntimeCommitProviderReceipt {
        provider: provider.to_owned(),
        commit_id: commit_id.to_owned(),
        finality,
    })
}

fn assert_submitted_call(calls: &ProviderCalls, expected_payload: &str, expected_key: &str) {
    let calls = calls.borrow();
    assert_eq!(
        calls.len(),
        1,
        "adapter must submit exactly one provider call"
    );
    assert_eq!(calls[0].0, expected_payload);
    assert_eq!(calls[0].1, expected_key);
}

fn assert_timeout_failure(request: &KolmeRuntimeCommitRequest) {
    let (provider, _calls) =
        RecordingProvider::with_result(Err(KolmeRuntimeCommitProviderError::Timeout));
    let mut client =
        AdapterBackedKolmeRuntimeCommitClient::new("kolme-local", provider).expect("client");
    assert_eq!(
        client.submit_commit(request),
        Err(KolmeRuntimeCommitError::ProviderTransport {
            kind: KolmeRuntimeCommitTransportErrorKind::Timeout,
            detail: "provider request timed out".to_owned(),
        })
    );
}

fn assert_provider_mismatch_failure(request: &KolmeRuntimeCommitRequest) {
    let (mut client, _calls) = client_with_receipt(
        "kolme-local",
        "kolme-remote",
        "kolme-commit:adapter-101",
        KolmeCommitReceiptFinality::Final,
    );
    assert_eq!(
        client.submit_commit(request),
        Err(KolmeRuntimeCommitError::ProviderMismatch {
            expected: "kolme-local".to_owned(),
            observed: "kolme-remote".to_owned(),
        })
    );
}

fn assert_non_final_failure(request: &KolmeRuntimeCommitRequest) {
    let (mut client, _calls) = client_with_receipt(
        "kolme-local",
        "kolme-local",
        "kolme-commit:adapter-102",
        KolmeCommitReceiptFinality::Pending,
    );
    assert_eq!(
        client.submit_commit(request),
        Err(KolmeRuntimeCommitError::NonFinalReceipt {
            commit_id: "kolme-commit:adapter-102".to_owned(),
            finality: KolmeCommitReceiptFinality::Pending,
        })
    );
}

fn assert_provider_mismatch_receipt_error(pipeline: &mut RuntimeCommitPipeline) {
    assert_eq!(
        pipeline.apply_receipt_finality(
            "op-sync-adapter-104",
            KolmeCommitReceiptFinality::Final,
            "kolme-remote",
            "kolme-commit:adapter-104",
        ),
        Err(KolmeRuntimeCommitError::ReceiptFieldMismatch {
            field: "receipt_provider",
            expected: "kolme-local".to_owned(),
            observed: "kolme-remote".to_owned(),
        })
    );
}
