use kamn_kolme::{normalize_broadcast_payload, KolmeBroadcastPayloadPolicyError};

#[test]
fn functional_normalize_broadcast_payload_maps_direct_signed_json() {
    let payload = "{\"message\":\"{\\\"pubkey\\\":\\\"pk\\\",\\\"nonce\\\":1,\\\"created\\\":\\\"2026-02-11T00:00:00Z\\\",\\\"messages\\\":[],\\\"max_height\\\":null}\",\"signature\":\"sig-direct\",\"recovery_id\":1}";
    let normalized = normalize_broadcast_payload(payload, "kolme-runtime-commit:direct:1")
        .expect("direct signed payload should normalize");
    assert!(normalized.contains("\"signature\":\"sig-direct\""));
    assert!(normalized.contains("\"recovery_id\":1"));
    assert!(normalized.contains("\\\"pubkey\\\":\\\"pk\\\""));
}

#[test]
fn functional_normalize_broadcast_payload_maps_signed_envelope_payload() {
    let payload = "{\"signer_key_id\":\"kamn:key:signer:1\",\"message\":\"operation_id=op\\nidempotency_key=abc\\n\",\"signature\":\"sig-envelope\",\"recovery_id\":1}";
    let normalized = normalize_broadcast_payload(payload, "abc")
        .expect("signed envelope payload should normalize");
    assert!(normalized.contains("\"signature\":\"sig-envelope\""));
    assert!(normalized.contains("\"recovery_id\":1"));
    assert!(normalized.contains("operation_id=op"));
    assert!(normalized.contains("idempotency_key=abc"));
}

#[test]
fn regression_issue_1757_normalize_broadcast_payload_rejects_empty_signer_key_id() {
    // Regression: #1757
    let payload = "{\"signer_key_id\":\"\",\"message\":\"operation_id=op\\nidempotency_key=abc\\n\",\"signature\":\"sig\",\"recovery_id\":1}";
    let error =
        normalize_broadcast_payload(payload, "abc").expect_err("empty signer_key_id must fail");
    assert_eq!(
        error,
        KolmeBroadcastPayloadPolicyError::MalformedResponse {
            reason: "field must not be empty: signer_key_id".to_owned(),
        }
    );
}

#[test]
fn regression_issue_1757_normalize_broadcast_payload_rejects_non_json_direct_message() {
    // Regression: #1757
    let payload = "{\"message\":\"operation_id=op\\nidempotency_key=abc\\n\",\"signature\":\"sig\",\"recovery_id\":1}";
    let error =
        normalize_broadcast_payload(payload, "abc").expect_err("non-json direct message must fail");
    assert_eq!(
        error,
        KolmeBroadcastPayloadPolicyError::MalformedResponse {
            reason: "direct signed payload message must be a JSON object string".to_owned(),
        }
    );
}
