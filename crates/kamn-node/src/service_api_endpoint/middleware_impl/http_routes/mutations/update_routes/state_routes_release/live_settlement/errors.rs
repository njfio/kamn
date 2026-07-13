use super::*;

pub(super) fn settlement_evidence_mismatch_error() -> Response {
    json_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal",
        "SETTLEMENT_EVIDENCE_MISMATCH",
        "confirmed settlement evidence does not match the durable intent",
    )
}

pub(super) fn settlement_outcome_ambiguous_error() -> Response {
    json_error(
        StatusCode::SERVICE_UNAVAILABLE,
        "unavailable",
        "SETTLEMENT_OUTCOME_AMBIGUOUS",
        "settlement submission outcome is ambiguous and requires reconciliation",
    )
}

pub(super) fn settlement_intent_conflict_error() -> Response {
    json_error(
        StatusCode::CONFLICT,
        "conflict",
        "SETTLEMENT_INTENT_CONFLICT",
        "settlement idempotency key conflicts with the durable intent",
    )
}

pub(super) fn settlement_transaction_expired_error() -> Response {
    json_error(
        StatusCode::CONFLICT,
        "conflict",
        "SETTLEMENT_TRANSACTION_EXPIRED",
        "persisted settlement transaction expired before confirmation",
    )
}

pub(super) fn invalid_release_key(message: &str) -> Response {
    json_error(
        StatusCode::BAD_REQUEST,
        "bad_request",
        "ESCROW_AGREEMENT_INVALID",
        message,
    )
}

fn json_error(status: StatusCode, error: &str, code: &str, message: &str) -> Response {
    super::super::super::payload::json_error_response(status, error, code, message)
}
