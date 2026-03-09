use super::*;

#[test]
fn integration_service_api_endpoint_accepts_case_variant_self_certifying_sender_did_binding() {
    let (snapshot, bind_addr, server) = start_service_api_server("127.0.0.1:34075", 1);
    let signer_public_key_hex =
        service_auth_public_key_hex_from_private_key_hex(TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX)
            .expect("service-auth public key should derive");
    let sender_did = format!("kamn:did:agent:pkh-{signer_public_key_hex}");
    let message_body = "{\"message\":\"hello\"}";
    let signature = service_api_request_signature_for_fields(
        sender_did.as_str(),
        1,
        service_api_request_state_hash(&snapshot).as_str(),
        message_body,
    );
    let response = send_http_request_with_headers_raw(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did.as_str()),
            ("X-KAMN-Request-Nonce", "1"),
            ("X-KAMN-Request-Signature", signature.as_str()),
            (
                "x-kamn-signer-public-key",
                signer_public_key_hex.to_uppercase().as_str(),
            ),
            ("X-KAMN-Authz-Scope", "messages:write"),
        ],
    );
    assert!(response.contains("HTTP/1.1 202 Accepted"));
    join_service_api_server(
        server,
        "service api endpoint should stop cleanly after case-variant sender DID auth flow",
    );
}

#[test]
fn regression_service_api_endpoint_rejects_legacy_sender_binding_without_signer_public_key_header() {
    let (snapshot, bind_addr, server) = start_service_api_server("127.0.0.1:34076", 1);
    let sender_did = "kamn:did:agent:legacy-auth-binding";
    let message_body = r#"{"recipient_did":"kamn:did:agent:legacy-auth-target","message":"hello"}"#;
    let signature = service_auth_sign_with_private_key_hex(
        sender_did,
        1,
        service_api_request_state_hash(&snapshot).as_str(),
        message_body,
        TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX,
    )
    .expect("raw legacy sender signature should derive");
    let response = send_http_request_with_headers_raw(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "1"),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    );
    assert!(response.contains("HTTP/1.1 401 Unauthorized"));
    let error_payload = parse_error_envelope_from_http_response(response.as_str());
    assert_eq!(
        error_payload.reason_code,
        "service_api_auth_signature_verification_failed"
    );
    assert!(
        error_payload.message.contains("x-kamn-signer-public-key"),
        "missing signer header rejection should explain the explicit signer binding contract"
    );
    join_service_api_server(
        server,
        "service api endpoint should stop cleanly after rejecting legacy auth fallback",
    );
}
