use super::*;
#[test]
fn integration_http_transport_submit_and_response_mapping() {
    let wire_payload = "operation_id=op-1\nstate_root=state-1\n";
    let idempotency_key = "kolme-runtime-commit:op-1:state-1:agent-1:1:payload-1";
    let response_body =
        "status=submitted\nprovider=kolme-local\ncommit_id=kolme-commit:1\nfinality=final\n";
    let base_url = spawn_single_request_server(
        response_body.to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains("POST /broadcast/runtime-commit HTTP/1.1"));
            assert!(request.contains("X-Idempotency-Key: "));
            assert!(request.ends_with(wire_payload));
        },
    );

    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        base_url.as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    let outcome = provider
        .submit_runtime_commit(wire_payload, idempotency_key)
        .expect("submit should succeed");
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => {
            assert_eq!(receipt.provider, "kolme-local");
            assert_eq!(receipt.commit_id, "kolme-commit:1");
            assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Final);
        }
        other => panic!("unexpected provider outcome: {other:?}"),
    }
}

#[test]
fn integration_http_transport_finality_query_and_response_mapping() {
    let commit_id = "commit:id/with space";
    let response_body =
        "{\"provider\":\"kolme-local\",\"commit_id\":\"commit:id/with space\",\"finality\":\"final\"}";
    let base_url = spawn_single_request_server(
        response_body.to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains(
                "GET /runtime-commit/status?commit_id=commit%3Aid%2Fwith%20space HTTP/1.1"
            ));
        },
    );

    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut checker = KolmeRuntimeCommitFinalityChecker::new(
        base_url.as_str(),
        "/runtime-commit/status",
        transport,
    )
    .expect("checker should build");

    let receipt = checker
        .check_commit_finality(commit_id)
        .expect("finality check should succeed");
    assert_eq!(receipt.provider, "kolme-local");
    assert_eq!(receipt.commit_id, commit_id);
    assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Final);
}

#[test]
fn regression_http_transport_timeout_maps_to_provider_timeout() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    thread::spawn(move || {
        let (_stream, _) = listener.accept().expect("connection should be accepted");
        thread::sleep(Duration::from_secs(2));
    });

    let transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        format!("http://{addr}").as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-1\n", "idempotency-key-1"),
        Err(KolmeRuntimeCommitProviderError::Timeout)
    );
}

#[test]
fn regression_http_transport_rejects_invalid_port_before_network_io() {
    let transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        "http://127.0.0.1:abc",
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-1\n", "idempotency-key-1"),
        Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "base_url port is invalid".to_owned(),
        })
    );
}

#[test]
fn regression_issue_1884_http_transport_rejects_empty_idempotency_key() {
    // Regression: #1884
    let transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        "http://127.0.0.1:1",
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-1\n", " "),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "idempotency_key must not be empty".to_owned(),
        })
    );
}

#[test]
fn regression_issue_1886_http_transport_rejects_empty_wire_payload() {
    // Regression: #1886
    let transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        "http://127.0.0.1:1",
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

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
    let idempotency_key = "kolme-runtime-commit:op-auth:state-auth:agent-1:1:payload-auth";
    let response_body =
        "status=submitted\nprovider=kolme-local\ncommit_id=kolme-commit:auth\nfinality=final\n";
    let base_url = spawn_single_request_server(
        response_body.to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains("Authorization: Bearer integration-token"));
        },
    );

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
        .submit_runtime_commit(wire_payload, idempotency_key)
        .expect("submit should succeed");
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => {
            assert_eq!(receipt.provider, "kolme-local");
            assert_eq!(receipt.commit_id, "kolme-commit:auth");
        }
        other => panic!("unexpected provider outcome: {other:?}"),
    }
}

