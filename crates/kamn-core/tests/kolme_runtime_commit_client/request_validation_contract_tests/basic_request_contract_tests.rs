use super::*;

#[test]
fn unit_commit_request_wire_payload_is_deterministic() {
    let first = KolmeRuntimeCommitRequest::deterministic(
        "op-sync-100",
        "state:abc123",
        "kamn:did:agent:runtime-node-1",
        7,
        "payload:stable",
    )
    .expect("first request should build");

    let second = KolmeRuntimeCommitRequest::deterministic(
        "op-sync-100",
        "state:abc123",
        "kamn:did:agent:runtime-node-1",
        7,
        "payload:stable",
    )
    .expect("second request should build");

    assert_eq!(first.idempotency_key(), second.idempotency_key());
    assert_eq!(first.to_wire_payload(), second.to_wire_payload());
}

#[test]
fn functional_in_memory_commit_client_returns_submitted_then_duplicate() {
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-sync-101",
        "state:abc123",
        "kamn:did:agent:runtime-node-2",
        2,
        "payload:functional",
    )
    .expect("request should build");

    let mut client =
        InMemoryKolmeRuntimeCommitClient::new("kolme-local").expect("client should build");
    let first = client
        .submit_commit(&request)
        .expect("first submit should succeed");
    let second = client
        .submit_commit(&request)
        .expect("duplicate submit should succeed");

    assert!(matches!(first, KolmeRuntimeCommitOutcome::Submitted(_)));
    assert!(matches!(second, KolmeRuntimeCommitOutcome::Duplicate(_)));
}
