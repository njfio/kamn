use super::super::*;
use crate::service_api_endpoint::middleware_impl::log_service_api_event_info;

pub(super) async fn log_request_received(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
) -> Result<(), Response> {
    match log_service_api_event_info(
        "service.api.request.received",
        &received_log_fields(parsed_request, correlation_id),
    ) {
        Ok(()) => Ok(()),
        Err(reason) => Err(super::internal_response(
            state,
            request_started_at,
            parsed_request,
            super::InternalResponseProjection {
                correlation_id,
                reason_code: REASON_CODE_REQUEST_LOG_EMISSION_FAILED,
                outcome: "log-error",
                error_label: "internal",
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: reason.as_str(),
            },
        )
        .await),
    }
}

fn received_log_fields<'a>(
    parsed_request: &'a ParsedRequest,
    correlation_id: &'a str,
) -> [(&'static str, &'a str); 3] {
    [
        ("correlation_id", correlation_id),
        ("method", parsed_request.method.as_str()),
        ("path", parsed_request.path.as_str()),
    ]
}

pub(super) async fn verify_request_identity(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
    replay_guard: &mut ServiceApiReplayGuard,
) -> Result<(), Response> {
    match super::auth::verify_service_api_request_identity(
        state.as_ref(),
        parsed_request,
        replay_guard,
    ) {
        Ok(()) => Ok(()),
        Err(error) => Err(auth_error_response(
            state,
            parsed_request,
            correlation_id,
            request_started_at,
            error,
        )
        .await),
    }
}

async fn auth_error_response(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
    error: RequestAuthFailure,
) -> Response {
    let (status_code, error_label, auth_error, outcome) = auth_error_details(error);
    super::internal_response(
        state,
        request_started_at,
        parsed_request,
        super::InternalResponseProjection {
            correlation_id,
            reason_code: auth_error.reason_code,
            outcome,
            error_label,
            status_code,
            message: auth_error.message.as_str(),
        },
    )
    .await
}

fn auth_error_details(
    error: RequestAuthFailure,
) -> (
    StatusCode,
    &'static str,
    ServiceApiReasonedError,
    &'static str,
) {
    match error {
        RequestAuthFailure::Unauthorized(reasoned_error) => (
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            reasoned_error,
            "unauthorized",
        ),
        RequestAuthFailure::Replay(reasoned_error) => {
            (StatusCode::CONFLICT, "replay", reasoned_error, "replay")
        }
    }
}

pub(super) async fn record_request_nonce(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
    replay_guard: &mut ServiceApiReplayGuard,
) -> Result<(), Response> {
    if let Err(error) = reserve_request_nonce(parsed_request, replay_guard) {
        return Err(auth_error_response(
            state,
            parsed_request,
            correlation_id,
            request_started_at,
            error,
        )
        .await);
    }
    let Some((sender_did, nonce)) = auth_nonce_persistence_input(parsed_request) else {
        return Ok(());
    };
    match persist_nonce_high_watermark(state, sender_did, nonce).await {
        Ok(()) => Ok(()),
        Err(error) => Err(nonce_persistence_response(
            state,
            parsed_request,
            correlation_id,
            request_started_at,
            error.as_str(),
        )
        .await),
    }
}

fn reserve_request_nonce(
    parsed_request: &ParsedRequest,
    replay_guard: &mut ServiceApiReplayGuard,
) -> Result<(), RequestAuthFailure> {
    super::auth::record_verified_service_api_request_nonce(parsed_request, replay_guard)
}

fn auth_nonce_persistence_input(parsed_request: &ParsedRequest) -> Option<(&str, u64)> {
    if !route_requires_auth(parsed_request.method.as_str(), parsed_request.path.as_str()) {
        return None;
    }
    let sender_did =
        super::auth::header_value(&parsed_request.headers, REQUEST_AUTH_SENDER_DID_HEADER)?;
    let nonce = super::auth::header_value(&parsed_request.headers, REQUEST_AUTH_NONCE_HEADER)?
        .parse::<u64>()
        .ok()?;
    Some((sender_did, nonce))
}

async fn persist_nonce_high_watermark(
    state: &Arc<ServiceApiRuntimeState>,
    sender_did: &str,
    nonce: u64,
) -> Result<(), String> {
    state
        .message_store
        .lock()
        .await
        .record_auth_nonce_high_watermark(sender_did, nonce)
        .map_err(|error| format!("service api auth nonce persistence failed: {error}"))
}

async fn nonce_persistence_response(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
    message: &str,
) -> Response {
    super::internal_response(
        state,
        request_started_at,
        parsed_request,
        super::InternalResponseProjection {
            correlation_id,
            reason_code: REASON_CODE_STATE_PERSISTENCE_FAILED,
            outcome: "persistence",
            error_label: "internal",
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message,
        },
    )
    .await
}
