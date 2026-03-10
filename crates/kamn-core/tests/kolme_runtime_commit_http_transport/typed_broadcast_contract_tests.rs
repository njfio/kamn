use super::*;
#[test]
fn integration_http_transport_submit_broadcast_request_put_and_parse_txhash() {
    let broadcast_request = KolmeApiBroadcastRequest::new("{\"nonce\":42}", "sig-42", 1)
        .expect("broadcast request should build");
    let idempotency_key = "kolme-runtime-commit:typed-broadcast-42";
    let base_url = spawn_single_request_server(
        "{\"txhash\":\"tx-typed-42\"}".to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains("PUT /broadcast HTTP/1.1"));
            assert!(request.contains("Content-Type: application/json"));
            assert!(request.contains("X-Idempotency-Key: kolme-runtime-commit:typed-broadcast-42"));
            assert!(request.contains("\"message\":\"{\\\"nonce\\\":42}\""));
            assert!(request.contains("\"signature\":\"sig-42\""));
            assert!(request.contains("\"recovery_id\":1"));
        },
    );

    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let response = transport
        .submit_broadcast_request(
            base_url.as_str(),
            "/broadcast",
            &broadcast_request,
            idempotency_key,
        )
        .expect("broadcast helper should succeed");
    assert_eq!(response.txhash, "tx-typed-42");
}

#[test]
fn regression_issue_1912_http_transport_submit_broadcast_trims_idempotency_key() {
    // Regression: #1912
    let broadcast_request = KolmeApiBroadcastRequest::new("{\"nonce\":42}", "sig-42", 1)
        .expect("broadcast request should build");
    let base_url = spawn_single_request_server(
        "{\"txhash\":\"tx-typed-1912\"}".to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains("PUT /broadcast HTTP/1.1"));
            assert!(
                request.contains("X-Idempotency-Key: kolme-runtime-commit:typed-broadcast-1912")
            );
        },
    );

    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let response = transport
        .submit_broadcast_request(
            base_url.as_str(),
            "/broadcast",
            &broadcast_request,
            "  kolme-runtime-commit:typed-broadcast-1912  ",
        )
        .expect("broadcast helper should normalize idempotency key");
    assert_eq!(response.txhash, "tx-typed-1912");
}

#[test]
fn regression_issue_1888_http_transport_submit_broadcast_defaults_empty_submit_path() {
    // Regression: #1888
    let broadcast_request = KolmeApiBroadcastRequest::new("{\"nonce\":8}", "sig-8", 1)
        .expect("broadcast request should build");
    let base_url = spawn_single_request_server(
        "{\"txhash\":\"tx-typed-8\"}".to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains("PUT /broadcast HTTP/1.1"));
        },
    );

    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let response = transport
        .submit_broadcast_request(
            base_url.as_str(),
            "   ",
            &broadcast_request,
            "kolme-runtime-commit:typed-broadcast-8",
        )
        .expect("broadcast helper should default empty submit path");
    assert_eq!(response.txhash, "tx-typed-8");
}

#[test]
fn regression_http_transport_submit_broadcast_request_rejects_malformed_txhash_response() {
    let broadcast_request = KolmeApiBroadcastRequest::new("{\"nonce\":7}", "sig-7", 1)
        .expect("broadcast request should build");
    let base_url = spawn_single_request_server(
        "{\"status\":\"ok\"}".to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains("PUT /broadcast HTTP/1.1"));
        },
    );

    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    assert_eq!(
        transport.submit_broadcast_request(
            base_url.as_str(),
            "/broadcast",
            &broadcast_request,
            "kolme-runtime-commit:typed-broadcast-7",
        ),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "missing required field: txhash".to_owned(),
        })
    );
}

