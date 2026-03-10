use super::support::*;
use super::*;

#[test]
fn regression_http_transport_fails_closed_on_content_length_mismatch() {
    let body = submitted_response("kolme-commit:1");
    let declared_length = body.len() + 9;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {declared_length}\r\nConnection: close\r\n\r\n{body}"
    );
    let base_url = spawn_server_with_raw_response(response, |request| {
        assert!(request.contains("POST /broadcast/runtime-commit HTTP/1.1"));
    });

    let error = provider(base_url.as_str(), "/broadcast/runtime-commit", 1)
        .submit_runtime_commit("operation_id=op-1\n", "idempotency-key-1")
        .expect_err("mismatch must fail closed");
    assert_eq!(
        error,
        KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: format!(
                "http response content-length mismatch: declared {declared_length}, observed {}",
                body.len()
            ),
        }
    );
}

#[test]
fn regression_http_transport_parses_connection_header_before_content_length() {
    let body = submitted_response("kolme-commit:ordered-headers");
    let response = format!(
        "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );
    let base_url = spawn_server_with_raw_response(response, |request| {
        assert!(request.contains("POST /broadcast/runtime-commit HTTP/1.1"));
    });

    let outcome = provider(base_url.as_str(), "/broadcast/runtime-commit", 1)
        .submit_runtime_commit("operation_id=ordered\n", "idempotency-key-ordered")
        .expect("response should parse");
    assert_submitted_receipt(outcome, "kolme-local", "kolme-commit:ordered-headers");
}

#[test]
fn regression_http_transport_parses_chunked_headers_without_early_failure() {
    let body = submitted_response("kolme-commit:chunked");
    let first = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection", body.len());
    let second = format!(": close\r\n\r\n{body}");
    let base_url = spawn_server_with_chunked_raw_response(
        first,
        second,
        Duration::from_millis(25),
        |request| assert!(request.contains("POST /broadcast/runtime-commit HTTP/1.1")),
    );

    let outcome = provider(base_url.as_str(), "/broadcast/runtime-commit", 1)
        .submit_runtime_commit("operation_id=chunked\n", "idempotency-key-chunked")
        .expect("chunked headers should parse");
    assert_submitted_receipt(outcome, "kolme-local", "kolme-commit:chunked");
}

#[test]
fn regression_http_transport_maps_401_to_authorization_unavailable_error() {
    let base_url = spawn_single_request_server(
        "{\"error\":\"unauthorized\"}".to_owned(),
        "HTTP/1.1 401 Unauthorized",
        |_| {},
    );
    let error = provider(base_url.as_str(), "/broadcast/runtime-commit", 1)
        .submit_runtime_commit("operation_id=op-1\n", "idempotency-key-1")
        .expect_err("401 must map to unavailable");
    assert_eq!(
        error,
        KolmeRuntimeCommitProviderError::Unavailable {
            reason: "http response status indicates authorization failure: 401".to_owned(),
        }
    );
}

#[test]
fn regression_http_transport_maps_422_to_invalid_request_malformed_error() {
    let base_url = spawn_single_request_server(
        "{\"error\":\"validation failed\"}".to_owned(),
        "HTTP/1.1 422 Unprocessable Entity",
        |_| {},
    );
    let error = provider(base_url.as_str(), "/broadcast/runtime-commit", 1)
        .submit_runtime_commit("operation_id=op-1\n", "idempotency-key-1")
        .expect_err("422 must map to malformed response");
    assert_eq!(
        error,
        KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "http response status indicates invalid request: 422".to_owned(),
        }
    );
}
