use super::*;

pub(super) fn missing_nonce_failure() -> RequestAuthFailure {
    RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
        REASON_CODE_AUTH_NONCE_HEADER_MISSING,
        format!("missing required header: {REQUEST_AUTH_NONCE_HEADER}"),
    ))
}

pub(super) fn invalid_nonce_failure() -> RequestAuthFailure {
    RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
        REASON_CODE_AUTH_NONCE_INVALID,
        format!("invalid request nonce header: {REQUEST_AUTH_NONCE_HEADER}"),
    ))
}

pub(super) fn missing_signature_failure() -> RequestAuthFailure {
    RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
        REASON_CODE_AUTH_SIGNATURE_HEADER_MISSING,
        format!("missing required header: {REQUEST_AUTH_SIGNATURE_HEADER}"),
    ))
}
