use super::*;

type AuthError = (u16, &'static str, &'static str, &'static str);

pub(crate) fn validate_auth(
    method: &str,
    path: &str,
    body: &str,
    headers: &BTreeMap<String, String>,
    replay_guard: &mut BTreeSet<(String, u64)>,
    expected_agent_sender_did: &str,
) -> Result<(), AuthError> {
    ensure_live_test_env();
    let did_value = sender_did(headers)?;
    let did = AgentDid::parse(did_value.clone()).map_err(|_| invalid_sender_did())?;
    let nonce = request_nonce(headers)?;
    verify_signature(&did, nonce, body, headers)?;
    verify_replay_guard(replay_guard, &did_value, nonce)?;
    verify_route_sender(method, path, did.as_str(), expected_agent_sender_did)?;
    verify_scope(method, path, headers)
}

fn sender_did(headers: &BTreeMap<String, String>) -> Result<String, AuthError> {
    let did = header_value(headers, "x-kamn-sender-did", missing_sender_did())?;
    AgentDid::parse(did.to_owned()).map_err(|_| invalid_sender_did())?;
    Ok(did.to_owned())
}

fn request_nonce(headers: &BTreeMap<String, String>) -> Result<u64, AuthError> {
    header_value(headers, "x-kamn-request-nonce", missing_nonce())?
        .parse::<u64>()
        .map_err(|_| invalid_nonce())
}

fn verify_signature(
    did: &AgentDid,
    nonce: u64,
    body: &str,
    headers: &BTreeMap<String, String>,
) -> Result<(), AuthError> {
    let signature = header_value(headers, "x-kamn-request-signature", missing_signature())?;
    let expected = service_signature_for_fields(did, nonce, CHAIN_ID, CHAIN_VERSION, body)
        .map_err(|_| signature_failed())?;
    if expected == signature {
        return Ok(());
    }
    Err(signature_failed())
}

fn verify_replay_guard(
    replay_guard: &mut BTreeSet<(String, u64)>,
    did: &str,
    nonce: u64,
) -> Result<(), AuthError> {
    if replay_guard.insert((did.to_owned(), nonce)) {
        return Ok(());
    }
    Err(replayed_nonce())
}

fn verify_route_sender(
    method: &str,
    path: &str,
    sender_did: &str,
    expected_agent_sender_did: &str,
) -> Result<(), AuthError> {
    if method == "GET" && path.starts_with("/v1/agents/") && sender_did != expected_agent_sender_did
    {
        return Err(agent_route_sender_mismatch());
    }
    Ok(())
}

fn verify_scope(
    method: &str,
    path: &str,
    headers: &BTreeMap<String, String>,
) -> Result<(), AuthError> {
    let Some(expected_scope) = route_scope_support::required_scope_for_route(method, path) else {
        return Ok(());
    };
    let scope = header_value(headers, REQUEST_AUTH_SCOPE_HEADER, missing_scope())?;
    if scope == expected_scope {
        return Ok(());
    }
    Err(scope_mismatch())
}

fn header_value<'a>(
    headers: &'a BTreeMap<String, String>,
    name: &str,
    error: AuthError,
) -> Result<&'a str, AuthError> {
    headers.get(name).map(|value| value.as_str()).ok_or(error)
}

fn missing_sender_did() -> AuthError {
    (
        401,
        "unauthorized",
        "service_api_auth_sender_did_header_missing",
        "missing required header: x-kamn-sender-did",
    )
}

fn invalid_sender_did() -> AuthError {
    (
        401,
        "unauthorized",
        "service_api_auth_sender_did_invalid",
        "invalid sender did",
    )
}

fn missing_nonce() -> AuthError {
    (
        401,
        "unauthorized",
        "service_api_auth_nonce_header_missing",
        "missing required header: x-kamn-request-nonce",
    )
}

fn invalid_nonce() -> AuthError {
    (
        401,
        "unauthorized",
        "service_api_auth_nonce_invalid",
        "invalid request nonce header: x-kamn-request-nonce",
    )
}

fn missing_signature() -> AuthError {
    (
        401,
        "unauthorized",
        "service_api_auth_signature_header_missing",
        "missing required header: x-kamn-request-signature",
    )
}

fn signature_failed() -> AuthError {
    (
        401,
        "unauthorized",
        "service_api_auth_signature_verification_failed",
        "signature verification failed for request envelope",
    )
}

fn replayed_nonce() -> AuthError {
    (
        401,
        "unauthorized",
        "service_api_auth_replay_nonce_detected",
        "request nonce replay detected for sender",
    )
}

fn agent_route_sender_mismatch() -> AuthError {
    (
        401,
        "unauthorized",
        "service_api_auth_sender_did_invalid",
        "agent route sender did mismatch",
    )
}

fn missing_scope() -> AuthError {
    (
        401,
        "unauthorized",
        "service_api_auth_scope_header_missing",
        "missing required header: x-kamn-authz-scope",
    )
}

fn scope_mismatch() -> AuthError {
    (
        401,
        "unauthorized",
        "service_api_auth_scope_route_mismatch",
        "scope route mismatch",
    )
}
