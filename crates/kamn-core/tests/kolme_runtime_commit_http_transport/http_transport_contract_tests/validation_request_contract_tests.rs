use super::*;
#[test]
fn unit_http_transport_rejects_zero_timeout_seconds() {
    assert!(
        matches!(
            KolmeRuntimeCommitHttpTransport::new(0),
            Err(kamn_core::KolmeRuntimeCommitError::InvalidRequest {
                field: "transport_timeout_seconds",
                reason: "must be positive",
            })
        ),
        "http transport timeout must be positive"
    );
}

#[test]
fn regression_http_transport_partial_eq_requires_timeout_and_authorization_match() {
    let transport_a = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let transport_b = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let transport_timeout_mismatch =
        KolmeRuntimeCommitHttpTransport::new(3).expect("transport should build");
    let transport_auth_alpha =
        KolmeRuntimeCommitHttpTransport::new_with_authorization(2, "Bearer alpha")
            .expect("transport should build");
    let transport_auth_alpha_clone =
        KolmeRuntimeCommitHttpTransport::new_with_authorization(2, "Bearer alpha")
            .expect("transport should build");
    let transport_auth_beta =
        KolmeRuntimeCommitHttpTransport::new_with_authorization(2, "Bearer beta")
            .expect("transport should build");

    assert_eq!(transport_a, transport_b);
    assert_ne!(transport_a, transport_timeout_mismatch);
    assert_ne!(transport_a, transport_auth_alpha);
    assert_eq!(transport_auth_alpha, transport_auth_alpha_clone);
    assert_ne!(transport_auth_alpha, transport_auth_beta);
}

#[test]
fn unit_http_transport_block_fetch_rejects_zero_height() {
    let mut transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    assert_eq!(
        transport.fetch_block_by_height("http://127.0.0.1:3030", "/block/{height}", 0),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "block height must be positive".to_owned(),
        })
    );
}

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
fn integration_http_transport_fetch_next_nonce_query_and_parse() {
    let nonce_request =
        KolmeApiNextNonceRequest::new("pub:key/with space").expect("request should build");
    let base_url = spawn_single_request_server(
        "{\"next_nonce\":42,\"account_id\":\"acc-42\"}".to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(
                request.contains("GET /get-next-nonce?pubkey=pub%3Akey%2Fwith%20space HTTP/1.1")
            );
        },
    );

    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let response = transport
        .fetch_next_nonce(base_url.as_str(), "/get-next-nonce", &nonce_request)
        .expect("nonce helper should succeed");
    assert_eq!(response.next_nonce, 42);
    assert_eq!(response.account_id.as_deref(), Some("acc-42"));
}

