use super::*;

#[test]
fn regression_submit_commit_fails_closed_for_mutated_invalid_request() {
    // Regression: #825
    let mut request = KolmeRuntimeCommitRequest::deterministic(
        "op-sync-102",
        "state:abc123",
        "kamn:did:agent:runtime-node-3",
        3,
        "payload:valid",
    )
    .expect("request should build");
    request.payload_hash.clear();

    let mut client =
        InMemoryKolmeRuntimeCommitClient::new("kolme-local").expect("client should build");
    assert_eq!(
        client.submit_commit(&request),
        Err(KolmeRuntimeCommitError::InvalidRequest {
            field: "payload_hash",
            reason: "must not be empty",
        })
    );
}

#[test]
fn regression_issue_1892_submit_commit_fails_closed_for_empty_operation_id() {
    // Regression: #1892
    let mut request = KolmeRuntimeCommitRequest::deterministic(
        "op-sync-1892-operation",
        "state:op",
        "kamn:did:agent:runtime-node-1892",
        13,
        "payload:op",
    )
    .expect("request should build");
    request.operation_id.clear();

    let mut client =
        InMemoryKolmeRuntimeCommitClient::new("kolme-local").expect("client should build");
    assert_eq!(
        client.submit_commit(&request),
        Err(KolmeRuntimeCommitError::InvalidRequest {
            field: "operation_id",
            reason: "must not be empty",
        })
    );
}

#[test]
fn regression_issue_1892_submit_commit_fails_closed_for_empty_state_root() {
    // Regression: #1892
    let mut request = KolmeRuntimeCommitRequest::deterministic(
        "op-sync-1892-state-root",
        "state:op",
        "kamn:did:agent:runtime-node-1892",
        14,
        "payload:state",
    )
    .expect("request should build");
    request.state_root.clear();

    let mut client =
        InMemoryKolmeRuntimeCommitClient::new("kolme-local").expect("client should build");
    assert_eq!(
        client.submit_commit(&request),
        Err(KolmeRuntimeCommitError::InvalidRequest {
            field: "state_root",
            reason: "must not be empty",
        })
    );
}

#[test]
fn regression_issue_1894_submit_commit_fails_closed_for_multiline_operation_id() {
    // Regression: #1894
    let mut request = KolmeRuntimeCommitRequest::deterministic(
        "op-sync-1894-multiline",
        "state:op",
        "kamn:did:agent:runtime-node-1894",
        15,
        "payload:op",
    )
    .expect("request should build");
    request.operation_id.push_str("\nwrapped");

    let mut client =
        InMemoryKolmeRuntimeCommitClient::new("kolme-local").expect("client should build");
    assert_eq!(
        client.submit_commit(&request),
        Err(KolmeRuntimeCommitError::InvalidRequest {
            field: "wire_payload",
            reason: "fields must be single-line",
        })
    );
}

#[test]
fn regression_issue_1896_signed_envelope_constructor_rejects_empty_fields() {
    // Regression: #1896
    assert_empty_envelope_field_rejected(" ", "operation_id=op\n", "sig-1", "signer_key_id");
    assert_empty_envelope_field_rejected("kamn:key:signer:1", " ", "sig-1", "signed_message");
    assert_empty_envelope_field_rejected(
        "kamn:key:signer:1",
        "operation_id=op\n",
        " ",
        "signature",
    );
}

#[test]
fn regression_issue_1900_submit_commit_fails_closed_for_zero_nonce() {
    // Regression: #1900
    let mut request = KolmeRuntimeCommitRequest::deterministic(
        "op-sync-1900-nonce",
        "state:nonce",
        "kamn:did:agent:runtime-node-1900",
        1,
        "payload:nonce",
    )
    .expect("request should build");
    request.nonce = 0;

    let mut client =
        InMemoryKolmeRuntimeCommitClient::new("kolme-local").expect("client should build");
    assert_eq!(
        client.submit_commit(&request),
        Err(KolmeRuntimeCommitError::InvalidRequest {
            field: "nonce",
            reason: "must be positive",
        })
    );
}

fn assert_empty_envelope_field_rejected(
    signer_key_id: &str,
    signed_message: &str,
    signature: &str,
    field: &'static str,
) {
    assert_eq!(
        KolmeRuntimeCommitSignedBroadcastEnvelope::new(signer_key_id, signed_message, signature, 1),
        Err(KolmeRuntimeCommitError::InvalidRequest {
            field,
            reason: "must not be empty",
        })
    );
}
