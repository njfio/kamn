use super::*;

#[test]
fn regression_issue_1930_live_provider_outcome_conversion_preserves_semantics() {
    // Regression: #1930
    assert_submitted_outcome_conversion();
    assert_rejected_outcome_conversion();
}

#[test]
fn functional_live_provider_maps_tx_hash_and_block_height_to_deterministic_commit_id() {
    let outcome = submit_outcome(
        r#"{"status":"submitted","provider":"kolme-fork-local","tx_hash":"ab12cd34","block_height":"42","finality":"confirmed"}"#,
        "op-live-provider-003",
        "kamn:did:agent:live-provider-3",
        57,
    );
    assert_eq!(
        outcome,
        expected_submitted_outcome(
            "kolme-fork-local",
            "kolme-commit:ab12cd34:h42",
            KolmeCommitReceiptFinality::Final
        )
    );
}

#[test]
fn regression_live_provider_normalizes_backend_finality_aliases() {
    // Regression: #1412
    let outcome = submit_outcome(
        r#"{"status":"duplicate","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34:h42","finality":"accepted"}"#,
        "op-live-provider-004",
        "kamn:did:agent:live-provider-4",
        58,
    );
    assert_eq!(
        outcome,
        expected_duplicate_outcome(
            "kolme-fork-local",
            "kolme-commit:ab12cd34:h42",
            KolmeCommitReceiptFinality::Pending
        )
    );
}

fn assert_submitted_outcome_conversion() {
    let submitted =
        KolmeRuntimeCommitProviderOutcome::from(KamnKolmeRuntimeProviderOutcome::Submitted {
            provider: "kolme-live".to_owned(),
            commit_id: "kolme-commit:ab12cd34".to_owned(),
            finality: kamn_kolme::KolmeCommitReceiptFinality::Final,
        });
    assert_eq!(
        submitted,
        expected_submitted_outcome(
            "kolme-live",
            "kolme-commit:ab12cd34",
            KolmeCommitReceiptFinality::Final,
        )
    );
}

fn assert_rejected_outcome_conversion() {
    let rejected =
        KolmeRuntimeCommitProviderOutcome::from(KamnKolmeRuntimeProviderOutcome::Rejected {
            reason: "duplicate idempotency".to_owned(),
        });
    assert_eq!(
        rejected,
        KolmeRuntimeCommitProviderOutcome::Rejected {
            reason: "duplicate idempotency".to_owned(),
        }
    );
}

fn submit_outcome(
    response: &str,
    operation_id: &str,
    actor_did: &str,
    nonce: u64,
) -> KolmeRuntimeCommitProviderOutcome {
    let (transport, _calls) = RecordingTransport::with_result(Ok(response.to_owned()));
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        "http://127.0.0.1:3030",
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");
    let request = build_request(operation_id, actor_did, nonce);
    provider
        .submit_runtime_commit(&request.to_wire_payload(), request.idempotency_key())
        .expect("provider should map response")
}

fn build_request(operation_id: &str, actor_did: &str, nonce: u64) -> KolmeRuntimeCommitRequest {
    KolmeRuntimeCommitRequest::deterministic(
        operation_id,
        "state:live",
        actor_did,
        nonce,
        "payload:live-provider",
    )
    .expect("request should build")
}

fn expected_submitted_outcome(
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

fn expected_duplicate_outcome(
    provider: &str,
    commit_id: &str,
    finality: KolmeCommitReceiptFinality,
) -> KolmeRuntimeCommitProviderOutcome {
    KolmeRuntimeCommitProviderOutcome::Duplicate(KolmeRuntimeCommitProviderReceipt {
        provider: provider.to_owned(),
        commit_id: commit_id.to_owned(),
        finality,
    })
}
