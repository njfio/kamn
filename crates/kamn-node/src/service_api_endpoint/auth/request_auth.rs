use super::support::header_value;
use super::*;

mod failures;
mod nonce;
mod sender_binding;
mod signature;

use failures::{invalid_nonce_failure, missing_nonce_failure, missing_signature_failure};
use nonce::{record_fresh_nonce, require_request_nonce, verify_fresh_nonce, verify_positive_nonce};
pub(super) use sender_binding::{
    resolve_signer_public_key_for_request, sender_did_matches_signer_public_key,
};
use signature::{request_signature_matches, signature_verification_failure};

#[cfg(test)]
pub(crate) fn authorize_service_api_request(
    state: &ServiceApiRuntimeState,
    request: &ParsedRequest,
    replay_guard: &mut ServiceApiReplayGuard,
) -> Result<(), RequestAuthFailure> {
    authorize_service_api_request_with_legacy_policy(state, request, replay_guard, false)
}

pub(crate) fn verify_service_api_request_identity(
    state: &ServiceApiRuntimeState,
    request: &ParsedRequest,
    replay_guard: &mut ServiceApiReplayGuard,
) -> Result<(), RequestAuthFailure> {
    if !super::route_requires_auth(request.method.as_str(), request.path.as_str()) {
        return Ok(());
    }
    let sender_did = require_valid_sender_did_header(request)?;
    let nonce = require_request_nonce(request)?;
    verify_positive_nonce(nonce)?;
    verify_binding_and_signature(state, request, sender_did, nonce, false)?;
    verify_fresh_nonce(replay_guard, sender_did, nonce)
}

pub(crate) fn record_verified_service_api_request_nonce(
    request: &ParsedRequest,
    replay_guard: &mut ServiceApiReplayGuard,
) -> Result<(), RequestAuthFailure> {
    if !super::route_requires_auth(request.method.as_str(), request.path.as_str()) {
        return Ok(());
    }
    let sender_did = require_valid_sender_did_header(request)?;
    let nonce = require_request_nonce(request)?;
    record_fresh_nonce(replay_guard, sender_did, nonce)
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

#[cfg(test)]
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
    let nonce = require_request_nonce(request)?;
    verify_request_auth_envelope(
        state,
        request,
        replay_guard,
        sender_did,
        nonce,
        allow_legacy_sender_binding,
    )
}

#[cfg(test)]
fn verify_request_auth_envelope(
    state: &ServiceApiRuntimeState,
    request: &ParsedRequest,
    replay_guard: &mut ServiceApiReplayGuard,
    sender_did: &str,
    nonce: u64,
    allow_legacy_sender_binding: bool,
) -> Result<(), RequestAuthFailure> {
    verify_positive_nonce(nonce)?;
    verify_binding_and_signature(
        state,
        request,
        sender_did,
        nonce,
        allow_legacy_sender_binding,
    )?;
    record_fresh_nonce(replay_guard, sender_did, nonce)
}

fn verify_binding_and_signature(
    state: &ServiceApiRuntimeState,
    request: &ParsedRequest,
    sender_did: &str,
    nonce: u64,
    allow_legacy_sender_binding: bool,
) -> Result<(), RequestAuthFailure> {
    let signature = require_signature(request)?;
    let signer_public_key_hex =
        verified_signer_public_key_hex(request, state, sender_did, allow_legacy_sender_binding)?;
    verify_request_signature(
        state,
        request,
        sender_did,
        nonce,
        signature,
        signer_public_key_hex,
    )
}

fn verified_signer_public_key_hex<'a>(
    request: &'a ParsedRequest,
    state: &'a ServiceApiRuntimeState,
    sender_did: &str,
    allow_legacy_sender_binding: bool,
) -> Result<&'a str, RequestAuthFailure> {
    let signer_public_key_hex = resolve_request_signer_public_key(
        &request.headers,
        state.auth_public_key_hex.as_deref(),
        allow_legacy_sender_binding,
    )?;
    verify_sender_binding(
        sender_did,
        signer_public_key_hex,
        allow_legacy_sender_binding,
    )?;
    Ok(signer_public_key_hex)
}

fn require_signature(request: &ParsedRequest) -> Result<&str, RequestAuthFailure> {
    header_value(&request.headers, REQUEST_AUTH_SIGNATURE_HEADER)
        .ok_or_else(missing_signature_failure)
}

fn resolve_request_signer_public_key<'a>(
    headers: &'a BTreeMap<String, String>,
    fallback_public_key_hex: Option<&'a str>,
    allow_legacy_sender_binding: bool,
) -> Result<&'a str, RequestAuthFailure> {
    resolve_signer_public_key_for_request(
        headers,
        fallback_public_key_hex,
        allow_legacy_sender_binding,
    )
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
    if request_signature_matches(
        state,
        request,
        sender_did,
        nonce,
        signature,
        signer_public_key_hex,
    ) {
        return Ok(());
    }
    Err(signature_verification_failure())
}
