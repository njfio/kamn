use super::*;
#[test]
fn integration_kolme_fork_signed_envelope_submit_maps_txhash_response() {
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-1506-http-a",
        "state:1506",
        "kamn:did:agent:http-1506-a",
        21,
        "payload:1506-http-a",
    )
    .expect("request should build");
    let signed_envelope = request
        .translate_to_signed_broadcast_envelope(
            "kamn:key:signer:http-1",
            request.to_wire_payload().as_str(),
            "sig-1506-http-a",
            1,
        )
        .expect("signed envelope should build");
    let wire_payload = signed_envelope.to_wire_payload();

    let base_url = spawn_single_request_server(
        "{\"txhash\":\"ab12cd34\"}".to_owned(),
        "HTTP/1.1 200 OK",
        move |raw_request| {
            assert!(raw_request.contains("PUT /broadcast HTTP/1.1"));
            assert!(raw_request.contains("Content-Type: application/json"));
            assert!(raw_request.contains("\"message\":\"operation_id=op-1506-http-a"));
            assert!(raw_request.contains("\"signature\":\"sig-1506-http-a\""));
            assert!(raw_request.contains("\"recovery_id\":1"));
        },
    );

    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        base_url.as_str(),
        "kolme-fork-local",
        transport,
    )
    .expect("provider should build");

    let outcome = provider
        .submit_runtime_commit(wire_payload.as_str(), request.idempotency_key())
        .expect("signed submit should succeed");
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => {
            assert_eq!(receipt.provider, "kolme-fork-local");
            assert_eq!(receipt.commit_id, "kolme-commit:ab12cd34");
            assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Pending);
        }
        other => panic!("unexpected provider outcome: {other:?}"),
    }
}

#[test]
fn integration_kolme_fork_direct_signed_payload_submit_maps_txhash_response() {
    let wire_payload = "{\"message\":\"{\\\"pubkey\\\":\\\"pk-direct\\\",\\\"nonce\\\":1,\\\"created\\\":\\\"2026-02-11T00:00:00Z\\\",\\\"messages\\\":[],\\\"max_height\\\":null}\",\"signature\":\"sig-direct\",\"recovery_id\":1}";
    let idempotency_key = "kolme-runtime-commit:direct-signed:1";

    let base_url = spawn_single_request_server(
        "{\"txhash\":\"ab12cd34\"}".to_owned(),
        "HTTP/1.1 200 OK",
        move |raw_request| {
            assert!(raw_request.contains("PUT /broadcast HTTP/1.1"));
            assert!(raw_request.contains("Content-Type: application/json"));
            assert!(raw_request.contains("\"signature\":\"sig-direct\""));
            assert!(raw_request.contains("\"recovery_id\":1"));
            assert!(raw_request.contains("\\\"pubkey\\\":\\\"pk-direct\\\""));
        },
    );

    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        base_url.as_str(),
        "kolme-fork-local",
        transport,
    )
    .expect("provider should build");

    let outcome = provider
        .submit_runtime_commit(wire_payload, idempotency_key)
        .expect("direct signed submit should succeed");
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => {
            assert_eq!(receipt.provider, "kolme-fork-local");
            assert_eq!(receipt.commit_id, "kolme-commit:ab12cd34");
            assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Pending);
        }
        other => panic!("unexpected provider outcome: {other:?}"),
    }
}

#[test]
fn regression_kolme_fork_signed_envelope_requires_signer_key_id() {
    // Regression: #1506
    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        "http://127.0.0.1:3030",
        "kolme-fork-local",
        transport,
    )
    .expect("provider should build");

    let malformed_envelope = "{\"signer_key_id\":\"\",\"message\":\"operation_id=op\\nidempotency_key=abc\\n\",\"signature\":\"sig\",\"recovery_id\":1}";
    assert_eq!(
        provider.submit_runtime_commit(malformed_envelope, "abc"),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "field must not be empty: signer_key_id".to_owned(),
        })
    );
}

#[test]
fn regression_kolme_fork_direct_signed_payload_requires_json_message_shape() {
    // Regression: #1516
    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        "http://127.0.0.1:3030",
        "kolme-fork-local",
        transport,
    )
    .expect("provider should build");

    let malformed_direct_payload =
        "{\"message\":\"operation_id=op\\nidempotency_key=abc\\n\",\"signature\":\"sig\",\"recovery_id\":1}";
    assert_eq!(
        provider.submit_runtime_commit(malformed_direct_payload, "abc"),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "direct signed payload message must be a JSON object string".to_owned(),
        })
    );
}

#[test]
fn regression_kolme_fork_direct_signed_payload_requires_core_transaction_keys() {
    // Regression: #1519
    let missing_key_cases = [
        (
            "pubkey",
            "{\"nonce\":1,\"created\":\"2026-02-11T00:00:00Z\",\"messages\":[],\"max_height\":null}",
        ),
        (
            "nonce",
            "{\"pubkey\":\"pk-direct\",\"created\":\"2026-02-11T00:00:00Z\",\"messages\":[],\"max_height\":null}",
        ),
        (
            "created",
            "{\"pubkey\":\"pk-direct\",\"nonce\":1,\"messages\":[],\"max_height\":null}",
        ),
        (
            "messages",
            "{\"pubkey\":\"pk-direct\",\"nonce\":1,\"created\":\"2026-02-11T00:00:00Z\",\"max_height\":null}",
        ),
    ];

    for (missing_field, message_json) in missing_key_cases {
        let wire_payload = format!(
            "{{\"message\":\"{}\",\"signature\":\"sig-direct\",\"recovery_id\":1}}",
            message_json.replace('\\', "\\\\").replace('\"', "\\\"")
        );
        let base_url = spawn_single_request_server(
            "{\"txhash\":\"ab12cd34\"}".to_owned(),
            "HTTP/1.1 200 OK",
            |_raw_request| {},
        );

        let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
        let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
            base_url.as_str(),
            "kolme-fork-local",
            transport,
        )
        .expect("provider should build");

        assert_eq!(
            provider.submit_runtime_commit(
                wire_payload.as_str(),
                "kolme-runtime-commit:direct-required-fields:1"
            ),
            Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: format!(
                    "direct signed payload message missing required field: {missing_field}"
                ),
            })
        );
    }
}

