use super::super::*;

#[test]
fn integration_service_api_endpoint_scope_policy_rejects_missing_invalid_and_mismatched_scopes() {
    let (snapshot, bind_addr, server) = start_service_api_server("127.0.0.1:34075", 4);
    let message_body = "{\"message\":\"scope-policy-check\"}";
    let sender_did = "kamn:did:agent:test-client-scope-policy";
    let bound_sender_did = test_service_api_sender_did(sender_did);
    let signer_public_key_hex = test_service_api_auth_public_key_hex();
    let state_hash = service_api_request_state_hash(&snapshot);
    let missing_scope_response = scope_policy_response(
        bind_addr.as_str(),
        message_body,
        &bound_sender_did,
        signer_public_key_hex.as_str(),
        sender_did,
        state_hash.as_str(),
        9101,
        None,
    );
    assert_rejection_reason(
        missing_scope_response.as_str(),
        SERVICE_API_AUTH_SCOPE_HEADER_MISSING_REASON_CODE,
    );

    let invalid_scope_response = scope_policy_response(
        bind_addr.as_str(),
        message_body,
        &bound_sender_did,
        signer_public_key_hex.as_str(),
        sender_did,
        state_hash.as_str(),
        9102,
        Some(""),
    );
    assert_rejection_reason(
        invalid_scope_response.as_str(),
        SERVICE_API_AUTH_SCOPE_INVALID_REASON_CODE,
    );

    let mismatch_scope_response = scope_policy_response(
        bind_addr.as_str(),
        message_body,
        &bound_sender_did,
        signer_public_key_hex.as_str(),
        sender_did,
        state_hash.as_str(),
        9103,
        Some("messages:read"),
    );
    assert_rejection_reason(
        mismatch_scope_response.as_str(),
        SERVICE_API_AUTH_SCOPE_ROUTE_MISMATCH_REASON_CODE,
    );

    let allowed_scope_response = scope_policy_response(
        bind_addr.as_str(),
        message_body,
        &bound_sender_did,
        signer_public_key_hex.as_str(),
        sender_did,
        state_hash.as_str(),
        9104,
        Some("messages:write"),
    );
    assert!(allowed_scope_response.contains("HTTP/1.1 202 Accepted"));
    join_service_api_server(
        server,
        "service api endpoint should stop cleanly after scope policy checks",
    );
}

#[test]
fn integration_service_api_endpoint_rejects_missing_request_auth_headers() {
    let (_snapshot, bind_addr, server) = start_service_api_server("127.0.0.1:34053", 1);
    let unauth_response = send_http_request(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        "{\"message\":\"hello\"}",
    );
    assert!(unauth_response.contains("HTTP/1.1 401 Unauthorized"));
    let unauth_payload = parse_error_envelope_from_http_response(unauth_response.as_str());
    assert_eq!(unauth_payload.error, "unauthorized");
    assert_eq!(
        unauth_payload.reason_code,
        "service_api_auth_sender_did_header_missing"
    );
    assert!(unauth_payload.message.contains("x-kamn-sender-did"));
    join_service_api_server(
        server,
        "service api endpoint should stop cleanly after configured request budget",
    );
}

fn assert_rejection_reason(response: &str, reason_code: &str) {
    assert!(response.contains("HTTP/1.1 401 Unauthorized"));
    let payload = parse_error_envelope_from_http_response(response);
    assert_eq!(payload.error, "unauthorized");
    assert_eq!(payload.reason_code, reason_code);
}

fn scope_policy_response(
    bind_addr: &str,
    message_body: &str,
    bound_sender_did: &str,
    signer_public_key_hex: &str,
    sender_did: &str,
    state_hash: &str,
    nonce: u64,
    scope: Option<&str>,
) -> String {
    let nonce_text = nonce.to_string();
    let signature =
        service_api_request_signature_for_fields(sender_did, nonce, state_hash, message_body);
    let mut headers = vec![
        ("X-KAMN-Sender-DID", bound_sender_did),
        ("X-KAMN-Request-Nonce", nonce_text.as_str()),
        ("X-KAMN-Request-Signature", signature.as_str()),
        ("X-KAMN-Signer-Public-Key", signer_public_key_hex),
    ];
    if let Some(scope_value) = scope {
        headers.push(("X-KAMN-Authz-Scope", scope_value));
    }
    send_http_request_with_headers_raw(
        bind_addr,
        "POST",
        "/v1/messages/send",
        message_body,
        headers.as_slice(),
    )
}
