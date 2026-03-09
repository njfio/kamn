use super::super::*;

#[test]
fn integration_service_api_endpoint_scope_policy_rejects_missing_invalid_and_mismatched_scopes() {
    let (snapshot, bind_addr, server) = start_service_api_server("127.0.0.1:34075", 4);
    let message_body = "{\"message\":\"scope-policy-check\"}";
    let sender_did = "kamn:did:agent:test-client-scope-policy";
    let bound_sender_did = test_service_api_sender_did(sender_did);
    let signer_public_key_hex = test_service_api_auth_public_key_hex();
    let state_hash = service_api_request_state_hash(&snapshot);

    let missing_scope_response = send_http_request_with_headers_raw(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", bound_sender_did.as_str()),
            ("X-KAMN-Request-Nonce", "9101"),
            ("X-KAMN-Request-Signature", service_api_request_signature_for_fields(sender_did, 9101, state_hash.as_str(), message_body).as_str()),
            ("X-KAMN-Signer-Public-Key", signer_public_key_hex.as_str()),
        ],
    );
    assert_rejection_reason(missing_scope_response.as_str(), SERVICE_API_AUTH_SCOPE_HEADER_MISSING_REASON_CODE);

    let invalid_scope_response = send_http_request_with_headers_raw(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", bound_sender_did.as_str()),
            ("X-KAMN-Request-Nonce", "9102"),
            ("X-KAMN-Request-Signature", service_api_request_signature_for_fields(sender_did, 9102, state_hash.as_str(), message_body).as_str()),
            ("X-KAMN-Signer-Public-Key", signer_public_key_hex.as_str()),
            ("X-KAMN-Authz-Scope", ""),
        ],
    );
    assert_rejection_reason(invalid_scope_response.as_str(), SERVICE_API_AUTH_SCOPE_INVALID_REASON_CODE);

    let mismatch_scope_response = send_http_request_with_headers_raw(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", bound_sender_did.as_str()),
            ("X-KAMN-Request-Nonce", "9103"),
            ("X-KAMN-Request-Signature", service_api_request_signature_for_fields(sender_did, 9103, state_hash.as_str(), message_body).as_str()),
            ("X-KAMN-Signer-Public-Key", signer_public_key_hex.as_str()),
            ("X-KAMN-Authz-Scope", "messages:read"),
        ],
    );
    assert_rejection_reason(mismatch_scope_response.as_str(), SERVICE_API_AUTH_SCOPE_ROUTE_MISMATCH_REASON_CODE);

    let allowed_scope_response = send_http_request_with_headers_raw(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", bound_sender_did.as_str()),
            ("X-KAMN-Request-Nonce", "9104"),
            ("X-KAMN-Request-Signature", service_api_request_signature_for_fields(sender_did, 9104, state_hash.as_str(), message_body).as_str()),
            ("X-KAMN-Signer-Public-Key", signer_public_key_hex.as_str()),
            ("X-KAMN-Authz-Scope", "messages:write"),
        ],
    );
    assert!(allowed_scope_response.contains("HTTP/1.1 202 Accepted"));
    join_service_api_server(server, "service api endpoint should stop cleanly after scope policy checks");
}

#[test]
fn integration_service_api_endpoint_rejects_missing_request_auth_headers() {
    let (_snapshot, bind_addr, server) = start_service_api_server("127.0.0.1:34053", 1);
    let unauth_response = send_http_request(bind_addr.as_str(), "POST", "/v1/messages/send", "{\"message\":\"hello\"}");
    assert!(unauth_response.contains("HTTP/1.1 401 Unauthorized"));
    let unauth_payload = parse_error_envelope_from_http_response(unauth_response.as_str());
    assert_eq!(unauth_payload.error, "unauthorized");
    assert_eq!(unauth_payload.reason_code, "service_api_auth_sender_did_header_missing");
    assert!(unauth_payload.message.contains("x-kamn-sender-did"));
    join_service_api_server(server, "service api endpoint should stop cleanly after configured request budget");
}

fn assert_rejection_reason(response: &str, reason_code: &str) {
    assert!(response.contains("HTTP/1.1 401 Unauthorized"));
    let payload = parse_error_envelope_from_http_response(response);
    assert_eq!(payload.error, "unauthorized");
    assert_eq!(payload.reason_code, reason_code);
}
