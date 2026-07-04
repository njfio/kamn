use super::*;

#[test]
fn integration_service_api_endpoint_accepts_case_variant_self_certifying_sender_did_binding() {
    let (snapshot, bind_addr, server, _env) = start_service_api_server("127.0.0.1:34075", 1);
    let response = case_variant_binding_response(&snapshot, bind_addr.as_str());
    assert!(response.contains("HTTP/1.1 202 Accepted"));
    join_service_api_server(
        server,
        "service api endpoint should stop cleanly after case-variant sender DID auth flow",
    );
}

#[test]
fn regression_service_api_endpoint_rejects_legacy_sender_binding_without_signer_public_key_header()
{
    let (snapshot, bind_addr, server, _env) = start_service_api_server("127.0.0.1:34076", 1);
    let response = legacy_binding_rejection_response(&snapshot, bind_addr.as_str());
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

fn case_variant_binding_response(
    snapshot: &crate::service_api_endpoint::ServiceApiSnapshot,
    bind_addr: &str,
) -> String {
    let signer_public_key_hex =
        service_auth_public_key_hex_from_private_key_hex(TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX)
            .expect("service-auth public key should derive");
    let sender_did = format!("kamn:did:agent:pkh-{signer_public_key_hex}");
    let message_body = "{\"message\":\"hello\"}";
    let signature = service_api_request_signature_for_fields(
        sender_did.as_str(),
        1,
        service_api_request_state_hash(snapshot).as_str(),
        message_body,
    );
    let signer_header = signer_public_key_hex.to_uppercase();
    send_http_request_with_headers_raw(
        bind_addr,
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did.as_str()),
            ("X-KAMN-Request-Nonce", "1"),
            ("X-KAMN-Request-Signature", signature.as_str()),
            ("x-kamn-signer-public-key", signer_header.as_str()),
            ("X-KAMN-Authz-Scope", "messages:write"),
        ],
    )
}

fn legacy_binding_rejection_response(
    snapshot: &crate::service_api_endpoint::ServiceApiSnapshot,
    bind_addr: &str,
) -> String {
    let sender_did = "kamn:did:agent:legacy-auth-binding";
    let message_body = r#"{"recipient_did":"kamn:did:agent:legacy-auth-target","message":"hello"}"#;
    let signature = service_auth_sign_with_private_key_hex(
        sender_did,
        1,
        service_api_request_state_hash(snapshot).as_str(),
        message_body,
        TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX,
    )
    .expect("raw legacy sender signature should derive");
    send_http_request_with_headers_raw(
        bind_addr,
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "1"),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    )
}
