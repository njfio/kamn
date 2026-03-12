use super::*;

pub(super) fn require_request_nonce(request: &ParsedRequest) -> Result<u64, RequestAuthFailure> {
    let nonce_raw = header_value(&request.headers, REQUEST_AUTH_NONCE_HEADER)
        .ok_or_else(missing_nonce_failure)?;
    nonce_raw
        .parse::<u64>()
        .map_err(|_| invalid_nonce_failure())
}

pub(super) fn verify_positive_nonce(nonce: u64) -> Result<(), RequestAuthFailure> {
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

pub(super) fn record_fresh_nonce(
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
