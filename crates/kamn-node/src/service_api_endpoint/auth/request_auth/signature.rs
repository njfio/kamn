use super::super::support::service_api_signature_state_hash;
use super::*;

pub(super) fn request_signature_matches(
    state: &ServiceApiRuntimeState,
    request: &ParsedRequest,
    sender_did: &str,
    nonce: u64,
    signature: &str,
    signer_public_key_hex: &str,
) -> bool {
    let state_hash = service_api_signature_state_hash(&state.snapshot);
    service_auth_verify_with_public_key_hex(
        signature,
        sender_did,
        nonce,
        state_hash.as_str(),
        request.body.as_str(),
        signer_public_key_hex,
    )
    .is_ok()
}

pub(super) fn signature_verification_failure() -> RequestAuthFailure {
    RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
        REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED,
        "signature verification failed for request envelope",
    ))
}
