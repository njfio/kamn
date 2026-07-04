use super::*;

#[path = "typed_broadcast_contract_tests/support.rs"]
mod support;

use support::*;

#[test]
fn integration_http_transport_submit_broadcast_request_put_and_parse_txhash() {
    let request = broadcast_request("{\"nonce\":42}", "sig-42", 1);
    let base_url = broadcast_server("tx-typed-42", |request| {
        assert!(request.contains("X-Idempotency-Key: kolme-runtime-commit:typed-broadcast-42"));
        assert!(request.contains("\"message\":\"{\\\"nonce\\\":42}\""));
        assert!(request.contains("\"signature\":\"sig-42\""));
        assert!(request.contains("\"recovery_id\":1"));
    });

    let txhash = submit_broadcast(
        base_url.as_str(),
        "/broadcast",
        &request,
        "kolme-runtime-commit:typed-broadcast-42",
    );
    assert_eq!(txhash, "tx-typed-42");
}

#[test]
fn regression_issue_1912_http_transport_submit_broadcast_trims_idempotency_key() {
    let request = broadcast_request("{\"nonce\":42}", "sig-42", 1);
    let base_url = broadcast_server("tx-typed-1912", |request| {
        assert!(request.contains("X-Idempotency-Key: kolme-runtime-commit:typed-broadcast-1912"));
    });

    let txhash = submit_broadcast(
        base_url.as_str(),
        "/broadcast",
        &request,
        "  kolme-runtime-commit:typed-broadcast-1912  ",
    );
    assert_eq!(txhash, "tx-typed-1912");
}

#[test]
fn regression_issue_1888_http_transport_submit_broadcast_defaults_empty_submit_path() {
    let request = broadcast_request("{\"nonce\":8}", "sig-8", 1);
    let base_url = broadcast_server("tx-typed-8", |request| {
        assert!(request.contains("PUT /broadcast HTTP/1.1"));
    });

    let txhash = submit_broadcast(
        base_url.as_str(),
        "   ",
        &request,
        "kolme-runtime-commit:typed-broadcast-8",
    );
    assert_eq!(txhash, "tx-typed-8");
}

#[test]
fn regression_http_transport_submit_broadcast_request_rejects_malformed_txhash_response() {
    let request = broadcast_request("{\"nonce\":7}", "sig-7", 1);
    let base_url = spawn_single_request_server(
        "{\"status\":\"ok\"}".to_owned(),
        "HTTP/1.1 200 OK",
        |request| assert!(request.contains("PUT /broadcast HTTP/1.1")),
    );

    let error = new_broadcast_transport()
        .submit_broadcast_request(
            base_url.as_str(),
            "/broadcast",
            &request,
            "kolme-runtime-commit:typed-broadcast-7",
        )
        .expect_err("missing txhash must fail");
    assert_eq!(
        error,
        KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "missing required field: txhash".to_owned(),
        }
    );
}
