use super::*;
#[test]
fn regression_http_transport_fails_closed_on_content_length_mismatch() {
    let body = "status=submitted\nprovider=kolme-local\ncommit_id=kolme-commit:1\nfinality=final\n";
    let declared_length = body.len() + 9;
    let raw_response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {declared_length}\r\nConnection: close\r\n\r\n{body}"
    );

    let base_url = spawn_server_with_raw_response(raw_response, |request| {
        assert!(request.contains("POST /broadcast/runtime-commit HTTP/1.1"));
    });

    let transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        base_url.as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-1\n", "idempotency-key-1"),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: format!(
                "http response content-length mismatch: declared {declared_length}, observed {}",
                body.len()
            ),
        })
    );
}

#[test]
fn regression_http_transport_parses_connection_header_before_content_length() {
    let response_body = "status=submitted\nprovider=kolme-local\ncommit_id=kolme-commit:ordered-headers\nfinality=final\n";
    let raw_response = format!(
        "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
        response_body.len(),
        response_body
    );
    let base_url = spawn_server_with_raw_response(raw_response, |request| {
        assert!(request.contains("POST /broadcast/runtime-commit HTTP/1.1"));
    });

    let transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        base_url.as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    let outcome = provider
        .submit_runtime_commit("operation_id=ordered\n", "idempotency-key-ordered")
        .expect("response with connection header before content-length should parse");
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => {
            assert_eq!(receipt.provider, "kolme-local");
            assert_eq!(receipt.commit_id, "kolme-commit:ordered-headers");
        }
        other => panic!("unexpected provider outcome: {other:?}"),
    }
}

#[test]
fn regression_http_transport_parses_chunked_headers_without_early_failure() {
    let response_body =
        "status=submitted\nprovider=kolme-local\ncommit_id=kolme-commit:chunked\nfinality=final\n";
    let first_chunk = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection",
        response_body.len()
    );
    let second_chunk = format!(": close\r\n\r\n{response_body}");
    let base_url = spawn_server_with_chunked_raw_response(
        first_chunk,
        second_chunk,
        Duration::from_millis(25),
        |request| {
            assert!(request.contains("POST /broadcast/runtime-commit HTTP/1.1"));
        },
    );

    let transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        base_url.as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    let outcome = provider
        .submit_runtime_commit("operation_id=chunked\n", "idempotency-key-chunked")
        .expect("chunked headers should parse once header boundary is complete");
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => {
            assert_eq!(receipt.provider, "kolme-local");
            assert_eq!(receipt.commit_id, "kolme-commit:chunked");
        }
        other => panic!("unexpected provider outcome: {other:?}"),
    }
}

#[test]
fn regression_http_transport_maps_401_to_authorization_unavailable_error() {
    let base_url = spawn_single_request_server(
        "{\"error\":\"unauthorized\"}".to_owned(),
        "HTTP/1.1 401 Unauthorized",
        |_| {},
    );

    let transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        base_url.as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-1\n", "idempotency-key-1"),
        Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "http response status indicates authorization failure: 401".to_owned(),
        })
    );
}

#[test]
fn regression_http_transport_maps_422_to_invalid_request_malformed_error() {
    let base_url = spawn_single_request_server(
        "{\"error\":\"validation failed\"}".to_owned(),
        "HTTP/1.1 422 Unprocessable Entity",
        |_| {},
    );

    let transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        base_url.as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-1\n", "idempotency-key-1"),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "http response status indicates invalid request: 422".to_owned(),
        })
    );
}

