use kamn_core::{
    InMemoryKolmeRuntimeCommitClient, KolmeCommitReceiptFinality, KolmeRuntimeCommitError,
    KolmeRuntimeCommitRequest, RuntimeCommitLifecycleState, RuntimeCommitPipeline,
};

fn request(operation_id: &str, nonce: u64) -> KolmeRuntimeCommitRequest {
    KolmeRuntimeCommitRequest::deterministic(
        operation_id,
        "state:runtime-finality",
        "kamn:did:agent:runtime-finality-node",
        nonce,
        format!("payload:{operation_id}:{nonce}").as_str(),
    )
    .expect("request should build")
}

#[test]
fn unit_finality_projection_counts_pending_final_and_failed_states() {
    let mut client =
        InMemoryKolmeRuntimeCommitClient::new("kolme-local").expect("client should build");
    let mut pipeline = RuntimeCommitPipeline::new();

    let pending = request("op-pending", 1);
    let final_commit = request("op-final", 2);
    let failed = request("op-failed", 3);

    client.set_finality_for_idempotency_key(
        final_commit.idempotency_key(),
        KolmeCommitReceiptFinality::Final,
    );
    client.reject_idempotency_key(failed.idempotency_key(), "fixture-reject");

    pipeline
        .submit_with_client(&mut client, pending)
        .expect("pending submit should succeed");
    pipeline
        .submit_with_client(&mut client, final_commit)
        .expect("final submit should succeed");
    pipeline
        .submit_with_client(&mut client, failed)
        .expect("failed submit should remain typed");

    let projection = pipeline.finality_projection();
    assert_eq!(projection.pending_count, 1);
    assert_eq!(projection.final_count, 1);
    assert_eq!(projection.failed_count, 1);
}

#[test]
fn functional_pending_commit_receipt_sets_requeue_until_finalized() {
    let mut client =
        InMemoryKolmeRuntimeCommitClient::new("kolme-local").expect("client should build");
    let mut pipeline = RuntimeCommitPipeline::new();

    let pending = request("op-requeue", 4);
    let submitted = pipeline
        .submit_with_client(&mut client, pending.clone())
        .expect("submit should succeed");
    assert_eq!(submitted.state, RuntimeCommitLifecycleState::Pending);
    assert!(submitted.needs_requeue);

    let finalized = pipeline
        .apply_receipt_finality(
            pending.operation_id.as_str(),
            KolmeCommitReceiptFinality::Final,
            "kolme-local",
            "receipt-final-op-requeue",
        )
        .expect("finality update should succeed");
    assert_eq!(finalized.state, RuntimeCommitLifecycleState::Finalized);
    assert!(!finalized.needs_requeue);
    assert_eq!(
        finalized.receipt_commit_id.as_deref(),
        Some("receipt-final-op-requeue")
    );
}

#[test]
fn integration_commit_to_receipt_flow_is_deterministic() {
    let mut client =
        InMemoryKolmeRuntimeCommitClient::new("kolme-local").expect("client should build");
    let mut pipeline = RuntimeCommitPipeline::new();

    let finalized_request = request("op-deterministic-final", 5);
    client.set_finality_for_idempotency_key(
        finalized_request.idempotency_key(),
        KolmeCommitReceiptFinality::Final,
    );
    let finalized = pipeline
        .submit_with_client(&mut client, finalized_request.clone())
        .expect("finalized submit should succeed");
    assert_eq!(finalized.state, RuntimeCommitLifecycleState::Finalized);

    let pending_then_failed = request("op-deterministic-failed", 6);
    let pending = pipeline
        .submit_with_client(&mut client, pending_then_failed.clone())
        .expect("pending submit should succeed");
    assert_eq!(pending.state, RuntimeCommitLifecycleState::Pending);
    let failed = pipeline
        .apply_receipt_finality(
            pending_then_failed.operation_id.as_str(),
            KolmeCommitReceiptFinality::Failed,
            "kolme-local",
            "receipt-failed-op-deterministic-failed",
        )
        .expect("failed finality update should succeed");
    assert_eq!(failed.state, RuntimeCommitLifecycleState::Failed);
    assert!(!failed.needs_requeue);

    let projection = pipeline.finality_projection();
    assert_eq!(projection.pending_count, 0);
    assert_eq!(projection.final_count, 1);
    assert_eq!(projection.failed_count, 1);
}

#[test]
fn regression_unknown_operation_finality_update_is_rejected() {
    // Regression: #826
    let mut pipeline = RuntimeCommitPipeline::new();
    assert_eq!(
        pipeline.apply_receipt_finality(
            "missing-operation",
            KolmeCommitReceiptFinality::Final,
            "kolme-local",
            "receipt-missing",
        ),
        Err(KolmeRuntimeCommitError::UnknownOperationId {
            operation_id: "missing-operation".to_owned(),
        })
    );
}

#[test]
fn regression_finalized_operation_cannot_regress_to_pending() {
    // Regression: #826
    let mut client =
        InMemoryKolmeRuntimeCommitClient::new("kolme-local").expect("client should build");
    let mut pipeline = RuntimeCommitPipeline::new();
    let request = request("op-no-regress", 7);

    client.set_finality_for_idempotency_key(
        request.idempotency_key(),
        KolmeCommitReceiptFinality::Final,
    );
    pipeline
        .submit_with_client(&mut client, request.clone())
        .expect("submit should succeed");

    assert_eq!(
        pipeline.apply_receipt_finality(
            request.operation_id.as_str(),
            KolmeCommitReceiptFinality::Pending,
            "kolme-local",
            "receipt-pending-op-no-regress",
        ),
        Err(KolmeRuntimeCommitError::InvalidFinalityTransition {
            from: "finalized",
            to: "pending",
        })
    );
}
