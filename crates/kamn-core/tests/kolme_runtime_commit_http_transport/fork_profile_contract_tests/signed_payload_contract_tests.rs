use super::support::*;
use super::*;

#[test]
fn integration_kolme_fork_signed_envelope_submit_maps_txhash_response() {
    let (wire_payload, idempotency_key) = signed_envelope_fixture();
    let base_url = fork_txhash_server("ab12cd34", assert_signed_envelope_request);

    let outcome = fork_provider(base_url.as_str(), "kolme-fork-local")
        .submit_runtime_commit(wire_payload.as_str(), idempotency_key.as_str())
        .expect("signed submit should succeed");
    assert_pending_fork_receipt(outcome, "kolme-fork-local", "ab12cd34");
}

#[test]
fn integration_kolme_fork_direct_signed_payload_submit_maps_txhash_response() {
    let base_url = fork_txhash_server("ab12cd34", |raw_request| {
        assert!(raw_request.contains("Content-Type: application/json"));
        assert!(raw_request.contains("\"signature\":\"sig-direct\""));
        assert!(raw_request.contains("\"recovery_id\":1"));
        assert!(raw_request.contains("\\\"pubkey\\\":\\\"pk-direct\\\""));
    });

    let outcome = fork_provider(base_url.as_str(), "kolme-fork-local")
        .submit_runtime_commit(
            "{\"message\":\"{\\\"pubkey\\\":\\\"pk-direct\\\",\\\"nonce\\\":1,\\\"created\\\":\\\"2026-02-11T00:00:00Z\\\",\\\"messages\\\":[],\\\"max_height\\\":null}\",\"signature\":\"sig-direct\",\"recovery_id\":1}",
            "kolme-runtime-commit:direct-signed:1",
        )
        .expect("direct signed submit should succeed");
    assert_pending_fork_receipt(outcome, "kolme-fork-local", "ab12cd34");
}

#[test]
fn regression_kolme_fork_signed_envelope_requires_signer_key_id() {
    let error = fork_provider("http://127.0.0.1:3030", "kolme-fork-local")
        .submit_runtime_commit(
            "{\"signer_key_id\":\"\",\"message\":\"operation_id=op\\nidempotency_key=abc\\n\",\"signature\":\"sig\",\"recovery_id\":1}",
            "abc",
        )
        .expect_err("missing signer key id must fail");
    assert_eq!(
        error,
        KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "field must not be empty: signer_key_id".to_owned(),
        }
    );
}

#[test]
fn regression_kolme_fork_direct_signed_payload_requires_json_message_shape() {
    let error = fork_provider("http://127.0.0.1:3030", "kolme-fork-local")
        .submit_runtime_commit(
            "{\"message\":\"operation_id=op\\nidempotency_key=abc\\n\",\"signature\":\"sig\",\"recovery_id\":1}",
            "abc",
        )
        .expect_err("non-json direct message must fail");
    assert_eq!(
        error,
        KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "direct signed payload message must be a JSON object string".to_owned(),
        }
    );
}

#[test]
fn regression_kolme_fork_direct_signed_payload_requires_core_transaction_keys() {
    for (missing_field, message_json) in required_field_cases() {
        let wire_payload = format!(
            "{{\"message\":\"{}\",\"signature\":\"sig-direct\",\"recovery_id\":1}}",
            message_json.replace('\\', "\\\\").replace('"', "\\\"")
        );
        let base_url = fork_txhash_server("ab12cd34", |_raw_request| {});
        let error = fork_provider(base_url.as_str(), "kolme-fork-local")
            .submit_runtime_commit(
                wire_payload.as_str(),
                "kolme-runtime-commit:direct-required-fields:1",
            )
            .expect_err("missing core field must fail");
        assert_eq!(
            error,
            KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: format!(
                    "direct signed payload message missing required field: {missing_field}"
                ),
            }
        );
    }
}

fn required_field_cases() -> [(&'static str, &'static str); 4] {
    [
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
    ]
}

fn signed_envelope_fixture() -> (String, String) {
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-1506-http-a",
        "state:1506",
        "kamn:did:agent:http-1506-a",
        21,
        "payload:1506-http-a",
    )
    .expect("request should build");
    let envelope = request
        .translate_to_signed_broadcast_envelope(
            "kamn:key:signer:http-1",
            request.to_wire_payload().as_str(),
            "sig-1506-http-a",
            1,
        )
        .expect("signed envelope should build");
    (
        envelope.to_wire_payload(),
        request.idempotency_key().to_owned(),
    )
}

fn assert_signed_envelope_request(raw_request: String) {
    assert!(raw_request.contains("Content-Type: application/json"));
    assert!(raw_request.contains("\"message\":\"operation_id=op-1506-http-a"));
    assert!(raw_request.contains("\"signature\":\"sig-1506-http-a\""));
    assert!(raw_request.contains("\"recovery_id\":1"));
}
