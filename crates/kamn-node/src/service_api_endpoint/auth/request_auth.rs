use super::support::{header_value, service_api_signature_state_hash};
use super::*;

mod sender_binding;

pub(super) use sender_binding::{
    resolve_signer_public_key_for_request, sender_did_matches_signer_public_key,
};

pub(crate) fn authorize_service_api_request(
    state: &ServiceApiRuntimeState,
    request: &ParsedRequest,
    replay_guard: &mut ServiceApiReplayGuard,
) -> Result<(), RequestAuthFailure> {
    authorize_service_api_request_with_legacy_policy(state, request, replay_guard, false)
}

pub(super) fn require_valid_sender_did_header(
    request: &ParsedRequest,
) -> Result<&str, RequestAuthFailure> {
    let sender_did =
        header_value(&request.headers, REQUEST_AUTH_SENDER_DID_HEADER).ok_or_else(|| {
            RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
                REASON_CODE_AUTH_SENDER_DID_HEADER_MISSING,
                format!("missing required header: {REQUEST_AUTH_SENDER_DID_HEADER}"),
            ))
        })?;
    AgentDid::parse(sender_did).map_err(|error| {
        RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
            REASON_CODE_AUTH_SENDER_DID_INVALID,
            format!("invalid sender did: {error}"),
        ))
    })?;
    Ok(sender_did)
}

pub(super) fn authorize_service_api_request_with_legacy_policy(
    state: &ServiceApiRuntimeState,
    request: &ParsedRequest,
    replay_guard: &mut ServiceApiReplayGuard,
    allow_legacy_sender_binding: bool,
) -> Result<(), RequestAuthFailure> {
    if !super::route_requires_auth(request.method.as_str(), request.path.as_str()) {
        return Ok(());
    }
    let sender_did = require_valid_sender_did_header(request)?;
    let nonce_raw = header_value(&request.headers, REQUEST_AUTH_NONCE_HEADER).ok_or_else(|| {
        RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
            REASON_CODE_AUTH_NONCE_HEADER_MISSING,
            format!("missing required header: {REQUEST_AUTH_NONCE_HEADER}"),
        ))
    })?;
    let nonce = nonce_raw.parse::<u64>().map_err(|_| {
        RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
            REASON_CODE_AUTH_NONCE_INVALID,
            format!("invalid request nonce header: {REQUEST_AUTH_NONCE_HEADER}"),
        ))
    })?;
    verify_request_auth_envelope(
        state,
        request,
        replay_guard,
        sender_did,
        nonce,
        allow_legacy_sender_binding,
    )
}

fn verify_request_auth_envelope(
    state: &ServiceApiRuntimeState,
    request: &ParsedRequest,
    replay_guard: &mut ServiceApiReplayGuard,
    sender_did: &str,
    nonce: u64,
    allow_legacy_sender_binding: bool,
) -> Result<(), RequestAuthFailure> {
    verify_positive_nonce(nonce)?;
    let signature =
        header_value(&request.headers, REQUEST_AUTH_SIGNATURE_HEADER).ok_or_else(|| {
            RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
                REASON_CODE_AUTH_SIGNATURE_HEADER_MISSING,
                format!("missing required header: {REQUEST_AUTH_SIGNATURE_HEADER}"),
            ))
        })?;
    let signer_public_key_hex = resolve_signer_public_key_for_request(
        &request.headers,
        state.auth_public_key_hex.as_deref(),
        allow_legacy_sender_binding,
    )?;
    verify_sender_binding(
        sender_did,
        signer_public_key_hex,
        allow_legacy_sender_binding,
    )?;
    verify_request_signature(
        state,
        request,
        sender_did,
        nonce,
        signature,
        signer_public_key_hex,
    )?;
    record_fresh_nonce(replay_guard, sender_did, nonce)
}

fn verify_positive_nonce(nonce: u64) -> Result<(), RequestAuthFailure> {
    if nonce == 0 {
        return Err(RequestAuthFailure::Unauthorized(
            ServiceApiReasonedError::new(
                REASON_CODE_AUTH_NONCE_NON_POSITIVE,
                format!("request nonce must be positive: {REQUEST_AUTH_NONCE_HEADER}"),
            ),
        ));
    }
    Ok(())
}

fn verify_sender_binding(
    sender_did: &str,
    signer_public_key_hex: &str,
    allow_legacy_sender_binding: bool,
) -> Result<(), RequestAuthFailure> {
    if sender_did_matches_signer_public_key(
        sender_did,
        signer_public_key_hex,
        allow_legacy_sender_binding,
    ) {
        return Ok(());
    }
    Err(RequestAuthFailure::Unauthorized(
        ServiceApiReasonedError::new(
            REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED,
            "sender did does not match signer public key binding contract",
        ),
    ))
}

fn verify_request_signature(
    state: &ServiceApiRuntimeState,
    request: &ParsedRequest,
    sender_did: &str,
    nonce: u64,
    signature: &str,
    signer_public_key_hex: &str,
) -> Result<(), RequestAuthFailure> {
    let state_hash = service_api_signature_state_hash(&state.snapshot);
    let crypto_verified = service_auth_verify_with_public_key_hex(
        signature,
        sender_did,
        nonce,
        state_hash.as_str(),
        request.body.as_str(),
        signer_public_key_hex,
    )
    .is_ok();
    if crypto_verified {
        return Ok(());
    }
    Err(RequestAuthFailure::Unauthorized(
        ServiceApiReasonedError::new(
            REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED,
            "signature verification failed for request envelope",
        ),
    ))
}

fn record_fresh_nonce(
    replay_guard: &mut ServiceApiReplayGuard,
    sender_did: &str,
    nonce: u64,
) -> Result<(), RequestAuthFailure> {
    if replay_guard.record_nonce_if_fresh(sender_did, nonce, Instant::now()) {
        return Ok(());
    }
    Err(RequestAuthFailure::Replay(ServiceApiReasonedError::new(
        REASON_CODE_AUTH_REPLAY_NONCE_DETECTED,
        "request nonce replay detected for sender",
    )))
}
