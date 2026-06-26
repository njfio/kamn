use super::support::*;
use super::*;

#[test]
fn integration_http_transport_submit_and_response_mapping() {
    let wire_payload = "operation_id=op-1\nstate_root=state-1\n";
    let base_url = status_server(submitted_response("kolme-commit:1"), move |request| {
        assert!(request.contains("POST /broadcast/runtime-commit HTTP/1.1"));
        assert!(request.contains("X-Idempotency-Key: "));
        assert!(request.ends_with(wire_payload));
    });

    let outcome = provider(base_url.as_str(), "/broadcast/runtime-commit", 2)
        .submit_runtime_commit(
            wire_payload,
            "kolme-runtime-commit:op-1:state-1:agent-1:1:payload-1",
        )
        .expect("submit should succeed");
    assert_submitted_receipt(outcome, "kolme-local", "kolme-commit:1");
}

#[test]
fn integration_http_transport_finality_query_and_response_mapping() {
    let commit_id = "commit:id/with space";
    let response = "{\"provider\":\"kolme-local\",\"commit_id\":\"commit:id/with space\",\"finality\":\"final\"}";
    let base_url = status_server(response.to_owned(), move |request| {
        assert!(request
            .contains("GET /runtime-commit/status?commit_id=commit%3Aid%2Fwith%20space HTTP/1.1"));
    });

    let receipt = checker(base_url.as_str(), "/runtime-commit/status")
        .check_commit_finality(commit_id)
        .expect("finality check should succeed");
    assert_eq!(receipt.provider, "kolme-local");
    assert_eq!(receipt.commit_id, commit_id);
    assert_finality_receipt(receipt.finality, KolmeCommitReceiptFinality::Final);
}

#[test]
fn regression_http_transport_timeout_maps_to_provider_timeout() {
    let base_url = timeout_listener_url(Duration::from_secs(2));
    let mut provider = provider(base_url.as_str(), "/broadcast/runtime-commit", 1);
    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-1\n", "idempotency-key-1"),
        Err(KolmeRuntimeCommitProviderError::Timeout)
    );
}

#[test]
fn regression_http_transport_rejects_invalid_port_before_network_io() {
    let mut provider = provider("http://127.0.0.1:abc", "/broadcast/runtime-commit", 1);
    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-1\n", "idempotency-key-1"),
        Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "base_url port is invalid".to_owned(),
        })
    );
}

#[test]
fn regression_issue_1884_http_transport_rejects_empty_idempotency_key() {
    let mut provider = provider("http://127.0.0.1:1", "/broadcast/runtime-commit", 1);
    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-1\n", " "),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "idempotency_key must not be empty".to_owned(),
        })
    );
}

#[test]
fn regression_issue_1886_http_transport_rejects_empty_wire_payload() {
    let mut provider = provider("http://127.0.0.1:1", "/broadcast/runtime-commit", 1);
    assert_eq!(
        provider.submit_runtime_commit(" ", "idempotency-key-1"),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "wire_payload must not be empty".to_owned(),
        })
    );
}

#[test]
fn functional_http_transport_includes_authorization_header_when_configured() {
    let wire_payload = "operation_id=op-auth\nstate_root=state-auth\n";
    let base_url = status_server(submitted_response("kolme-commit:auth"), move |request| {
        assert!(request.contains("Authorization: Bearer integration-token"));
    });

    let transport =
        KolmeRuntimeCommitHttpTransport::new_with_authorization(2, "Bearer integration-token")
            .expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        base_url.as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");
    let outcome = provider
        .submit_runtime_commit(
            wire_payload,
            "kolme-runtime-commit:op-auth:state-auth:agent-1:1:payload-auth",
        )
        .expect("submit should succeed");
    assert_submitted_receipt(outcome, "kolme-local", "kolme-commit:auth");
}
