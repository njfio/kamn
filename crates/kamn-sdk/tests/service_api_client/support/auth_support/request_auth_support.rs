use super::*;

pub(crate) fn validate_auth(
    method: &str,
    path: &str,
    body: &str,
    headers: &BTreeMap<String, String>,
    replay_guard: &mut BTreeSet<(String, u64)>,
) -> Result<(), (u16, &'static str, &'static str, &'static str)> {
    ensure_test_service_auth_private_key();
    let did = sender_did(headers)?;
    let nonce = request_nonce(headers)?;
    verify_replay_guard(replay_guard, did.as_str(), nonce)?;
    verify_signature(did.as_str(), nonce, body, headers)?;
    verify_scope(method, path, headers)?;
    Ok(())
}

pub(crate) fn auth_with_scope(
    sender: &AgentDid,
    nonce: u64,
    body: &str,
    scope: &str,
) -> ServiceRequestAuth {
    ensure_test_service_auth_private_key();
    ServiceRequestAuth::new_with_scope(
        sender.clone(),
        nonce,
        service_signature_for_fields(sender, nonce, CHAIN_ID, CHAIN_VERSION, body)
            .expect("service signature should build"),
        Some(scope),
    )
    .expect("request auth with scope should build")
}

fn sender_did(headers: &BTreeMap<String, String>) -> Result<String, AuthError> {
    let did = header_value(headers, "x-kamn-sender-did", missing_sender_did())?;
    AgentDid::parse(did).map_err(|_| invalid_sender_did())?;
    Ok(did.to_owned())
}

fn request_nonce(headers: &BTreeMap<String, String>) -> Result<u64, AuthError> {
    let nonce = header_value(headers, "x-kamn-request-nonce", missing_nonce())?
        .parse::<u64>()
        .map_err(|_| invalid_nonce())?;
    if nonce == 0 {
        return Err(non_positive_nonce());
    }
    Ok(nonce)
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

fn verify_signature(
    did: &str,
    nonce: u64,
    body: &str,
    headers: &BTreeMap<String, String>,
) -> Result<(), AuthError> {
    let signature = header_value(headers, "x-kamn-request-signature", missing_signature())?;
    let sender = AgentDid::parse(did).map_err(|_| invalid_sender_did())?;
    let expected = service_signature_for_fields(&sender, nonce, CHAIN_ID, CHAIN_VERSION, body)
        .map_err(|_| signature_failed())?;
    if expected == signature {
        return Ok(());
    }
    Err(signature_failed())
}

fn verify_scope(
    method: &str,
    path: &str,
    headers: &BTreeMap<String, String>,
) -> Result<(), AuthError> {
    let Some(expected_scope) = required_scope_for_route(method, path) else {
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

type AuthError = (u16, &'static str, &'static str, &'static str);

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

fn non_positive_nonce() -> AuthError {
    (
        401,
        "unauthorized",
        "service_api_auth_nonce_non_positive",
        "request nonce must be positive: x-kamn-request-nonce",
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
        409,
        "replay",
        "service_api_auth_replay_nonce_detected",
        "request nonce replay detected for sender",
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
