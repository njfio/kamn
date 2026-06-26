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
    let timeout_mismatch = KolmeRuntimeCommitHttpTransport::new(3).expect("transport should build");
    let auth_alpha = KolmeRuntimeCommitHttpTransport::new_with_authorization(2, "Bearer alpha")
        .expect("transport should build");
    let auth_alpha_clone =
        KolmeRuntimeCommitHttpTransport::new_with_authorization(2, "Bearer alpha")
            .expect("transport should build");
    let auth_beta = KolmeRuntimeCommitHttpTransport::new_with_authorization(2, "Bearer beta")
        .expect("transport should build");

    assert_eq!(transport_a, transport_b);
    assert_ne!(transport_a, timeout_mismatch);
    assert_ne!(transport_a, auth_alpha);
    assert_eq!(auth_alpha, auth_alpha_clone);
    assert_ne!(auth_alpha, auth_beta);
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
