use super::support::*;

#[test]
fn regression_service_api_client_rejects_crlf_route_identifier_payload() {
    ensure_test_service_auth_private_key();
    let client = ServiceApiClient::connect("http://127.0.0.1:1").expect("client should construct");
    let sender = AgentDid::parse("kamn:did:agent:sdk-route-injection").expect("did");
    let auth = auth_with_scope(&sender, 1, "", "messages:read");
    let error = client
        .get_message("msg-1\r\nx-injected-header: true", &auth)
        .expect_err("crlf payload must fail closed before request emission");
    assert_eq!(
        error,
        SdkError::InvalidInput {
            field: "message_id",
            reason: "contains characters not allowed in route segment",
        }
    );
}

#[test]
fn regression_service_request_auth_rejects_crlf_signature_payload() {
    let sender = AgentDid::parse("kamn:did:agent:sdk-header-injection").expect("did");
    let error = ServiceRequestAuth::new_with_scope(
        sender,
        1,
        "sig\r\nx-injected-header: true".to_owned(),
        Some("messages:write"),
    )
    .expect_err("signature header injection payload must fail closed");
    assert_eq!(
        error,
        SdkError::InvalidInput {
            field: "request_auth.signature",
            reason: "contains invalid http header characters",
        }
    );
}

#[test]
fn regression_service_api_client_rejects_legacy_agent_profile_did() {
    ensure_test_service_auth_private_key();
    let client = ServiceApiClient::connect("http://127.0.0.1:1").expect("client should construct");
    let sender = AgentDid::parse("kamn:did:agent:sdk-legacy-agent-profile").expect("did");
    let auth = auth_with_scope(&sender, 1, "", "agents:read");
    let error = client
        .get_agent_profile("did:kamn:agent:alice", &auth)
        .expect_err("legacy did must fail closed before request emission");
    assert_eq!(
        error,
        SdkError::InvalidInput {
            field: "did",
            reason: "must start with kamn:did:agent:",
        }
    );
}

#[test]
fn regression_service_api_client_rejects_crlf_agent_did_route_payload() {
    ensure_test_service_auth_private_key();
    let client = ServiceApiClient::connect("http://127.0.0.1:1").expect("client should construct");
    let sender = AgentDid::parse("kamn:did:agent:sdk-did-route-injection").expect("did");
    let auth = auth_with_scope(&sender, 1, "", "agents:read");
    let error = client
        .get_agent_profile("kamn:did:agent:alice\r\nx-injected-header: true", &auth)
        .expect_err("crlf did payload must fail closed before request emission");
    assert_eq!(
        error,
        SdkError::InvalidInput {
            field: "did",
            reason: "contains characters not allowed in route segment",
        }
    );
}

#[test]
fn regression_service_request_auth_rejects_crlf_scope_payload() {
    let sender = AgentDid::parse("kamn:did:agent:sdk-scope-injection").expect("did");
    let error = ServiceRequestAuth::new_with_scope(
        sender,
        1,
        "sig:ok".to_owned(),
        Some("messages:read\r\nx-injected-header: true"),
    )
    .expect_err("scope header injection payload must fail closed");
    assert_eq!(
        error,
        SdkError::InvalidInput {
            field: "request_auth.scope",
            reason: "contains invalid http header characters",
        }
    );
}
