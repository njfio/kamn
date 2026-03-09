use super::*;

#[test]
fn integration_service_api_endpoint_rejects_legacy_deterministic_signature_profile() {
    let (snapshot, bind_addr, server) = start_service_api_server("127.0.0.1:34079", 1);
    let sender_did = "kamn:did:agent:test-client-legacy-signature";
    let payload = r#"{"message":"legacy-signature"}"#;
    let legacy_signature = kamn_core::legacy_signature_for_fields(
        sender_did,
        1,
        service_api_request_state_hash(&snapshot).as_str(),
        payload,
    );
    let response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        payload,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "1"),
            ("X-KAMN-Request-Signature", legacy_signature.as_str()),
        ],
    );
    assert!(response.contains("HTTP/1.1 401 Unauthorized"));
    let error_payload = parse_error_envelope_from_http_response(response.as_str());
    assert_eq!(
        error_payload.reason_code,
        "service_api_auth_signature_verification_failed"
    );
    join_service_api_server(
        server,
        "service api endpoint should stop cleanly after configured request budget",
    );
}

#[test]
fn regression_service_api_endpoint_rejects_legacy_signature_when_toggle_env_is_true() {
    let _legacy_toggle_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_AUTH_ALLOW_LEGACY_SIGNATURES",
        Some("true"),
    );
    let (snapshot, bind_addr, server) = start_service_api_server("127.0.0.1:34095", 1);
    let sender_did = "kamn:did:agent:test-client-legacy-toggle-true";
    let payload = r#"{"message":"legacy-signature-toggle-true"}"#;
    let legacy_signature = kamn_core::legacy_signature_for_fields(
        sender_did,
        1,
        service_api_request_state_hash(&snapshot).as_str(),
        payload,
    );
    let response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        payload,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "1"),
            ("X-KAMN-Request-Signature", legacy_signature.as_str()),
        ],
    );
    assert!(response.contains("HTTP/1.1 401 Unauthorized"));
    let error_payload = parse_error_envelope_from_http_response(response.as_str());
    assert_eq!(
        error_payload.reason_code,
        "service_api_auth_signature_verification_failed"
    );
    join_service_api_server(
        server,
        "service api endpoint should stop cleanly after configured request budget",
    );
}
