use super::super::*;

#[test]
fn unit_kolme_live_native_direct_message_contains_required_fields() {
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-node-live-2207",
        "state:node-live-2207",
        "kamn:did:agent:node-live-2207",
        1,
        "payload:node-live-2207",
    )
    .expect("request should build");

    let message = render_kolme_live_native_direct_message(
        &request,
        "02aa55bb66cc77dd88ee99ff00112233445566778899aabbccddeeff0011223344",
        19,
    )
    .expect("native direct message should render");

    assert!(message.contains(
        "\"pubkey\":\"02aa55bb66cc77dd88ee99ff00112233445566778899aabbccddeeff0011223344\""
    ));
    assert!(message.contains("\"nonce\":19"));
    assert!(message.contains("\"created\":\""));
    assert!(message.contains("\"messages\":["));
}

#[test]
fn integration_kolme_live_nonce_resolver_fetches_next_nonce() {
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":27,"account_id":"acct-2207"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let signer_adapter = KolmeForkSecp256k1SignerAdapter::from_private_key_hex(
        TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX,
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
    )
    .expect("deterministic signer adapter should build");
    let pubkey = signer_adapter.public_key_compressed_hex();

    let nonce = resolve_kolme_live_nonce(base_url.as_str(), &mut transport, pubkey.as_str())
        .expect("nonce should resolve");
    assert_eq!(nonce, 27);

    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(recorded_requests.len(), 1);
    assert!(recorded_requests[0].contains("GET /get-next-nonce?pubkey="));
}

#[test]
fn integration_kolme_live_nonce_resolver_retries_unavailable_then_succeeds() {
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![
        MockHttpReply {
            status_line: "HTTP/1.1 503 Service Unavailable",
            body: "{\"error\":\"nonce unavailable\"}".to_owned(),
        },
        MockHttpReply::ok(r#"{"next_nonce":29,"account_id":"acct-2207"}"#),
    ]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let pubkey = "03c9e9fd7028a8b17f4fbe0f6f7d38af2ec527f6bb2af04d4d2e2b0eb4f1f01b8a";

    let nonce = resolve_kolme_live_nonce(base_url.as_str(), &mut transport, pubkey)
        .expect("nonce resolver should recover from transient unavailable response");
    assert_eq!(nonce, 29);

    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        2,
        "nonce resolver should retry once after unavailable response"
    );
}

#[test]
fn regression_kolme_live_nonce_resolver_rejects_malformed_response() {
    // Regression: #2207
    let (base_url, requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(
        r#"{"next_nonce":0,"account_id":"acct-2207"}"#,
    )]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let error = resolve_kolme_live_nonce(
        base_url.as_str(),
        &mut transport,
        "02aa55bb66cc77dd88ee99ff00112233445566778899aabbccddeeff0011223344",
    )
    .expect_err("invalid nonce payload must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("nonce response malformed")),
        "expected fail-closed nonce parser error"
    );
    let recorded_requests = requests.lock().expect("request mutex should lock");
    assert_eq!(
        recorded_requests.len(),
        1,
        "malformed nonce responses must fail fast without retry"
    );
}
