use super::super::*;

mod transaction_authorization;

pub(super) use transaction_authorization::enforce_transaction_authorization;

pub(super) async fn enforce_scope_policy(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
) -> Result<(), Response> {
    match super::auth::enforce_request_scope_policy(parsed_request) {
        Ok(()) => Ok(()),
        Err(error) => Err(super::internal_response(
            state,
            request_started_at,
            parsed_request,
            super::InternalResponseProjection {
                correlation_id,
                reason_code: error.reason_code,
                outcome: "unauthorized",
                error_label: "unauthorized",
                status_code: StatusCode::UNAUTHORIZED,
                message: error.message.as_str(),
            },
        )
        .await),
    }
}

pub(super) async fn enforce_sender_anti_spam(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
) -> Result<(), Response> {
    match super::auth::enforce_sender_anti_spam(state, parsed_request).await {
        Ok(()) => Ok(()),
        Err(error) => Err(anti_spam_response(
            state,
            parsed_request,
            correlation_id,
            request_started_at,
            error,
        )
        .await),
    }
}

pub(super) async fn enforce_ingress_rate_limit(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
) -> Result<(), Response> {
    if !route_requires_auth(parsed_request.method.as_str(), parsed_request.path.as_str()) {
        return Ok(());
    }
    let mut ingress_rate_window = state.ingress_rate_window.lock().await;
    if ingress_rate_window.try_record_request(Instant::now()) {
        return Ok(());
    }
    Err(
        ingress_rate_limit_response(state, parsed_request, correlation_id, request_started_at)
            .await,
    )
}

pub(super) async fn validate_websocket_requirements(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
    is_websocket_route: bool,
) -> Result<(), Response> {
    match super::websocket::validate_websocket_route_requirements(
        is_websocket_route,
        &parsed_request.headers,
    ) {
        Ok(()) => Ok(()),
        Err(error) => Err(websocket_requirement_response(
            state,
            parsed_request,
            correlation_id,
            request_started_at,
            error,
        )
        .await),
    }
}

async fn anti_spam_response(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
    error: ServiceApiReasonedError,
) -> Response {
    let projection = anti_spam_projection(error.reason_code);
    super::internal_response(
        state,
        request_started_at,
        parsed_request,
        super::InternalResponseProjection {
            correlation_id,
            reason_code: projection.reason_code,
            outcome: projection.outcome,
            error_label: projection.error_label,
            status_code: projection.status_code,
            message: error.message.as_str(),
        },
    )
    .await
}

fn anti_spam_projection(reason_code: &'static str) -> ServiceApiLifecycleRejectionPolicy {
    service_api_lifecycle_rejection_policy(reason_code).unwrap_or(
        ServiceApiLifecycleRejectionPolicy {
            rejection_class: LIFECYCLE_REJECTION_CLASS_SENDER_ADMISSION,
            reason_code,
            status_code: StatusCode::TOO_MANY_REQUESTS,
            error_label: "too-many-requests",
            outcome: "anti-spam",
            default_message: "sender request rejected by anti-spam policy",
        },
    )
}

async fn ingress_rate_limit_response(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
) -> Response {
    let projection = ingress_rate_limit_projection();
    super::internal_response(
        state,
        request_started_at,
        parsed_request,
        super::InternalResponseProjection {
            correlation_id,
            reason_code: projection.reason_code,
            outcome: projection.outcome,
            error_label: projection.error_label,
            status_code: projection.status_code,
            message: projection.default_message,
        },
    )
    .await
}

fn ingress_rate_limit_projection() -> ServiceApiLifecycleRejectionPolicy {
    service_api_lifecycle_rejection_policy(REASON_CODE_INGRESS_RATE_LIMIT_EXCEEDED).unwrap_or(
        ServiceApiLifecycleRejectionPolicy {
            rejection_class: LIFECYCLE_REJECTION_CLASS_ASYNC_LIMITER,
            reason_code: REASON_CODE_INGRESS_RATE_LIMIT_EXCEEDED,
            status_code: StatusCode::TOO_MANY_REQUESTS,
            error_label: "too-many-requests",
            outcome: "rate-limit",
            default_message: "ingress rate limit exceeded",
        },
    )
}

async fn websocket_requirement_response(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
    error: ServiceApiReasonedError,
) -> Response {
    super::internal_response(
        state,
        request_started_at,
        parsed_request,
        super::InternalResponseProjection {
            correlation_id,
            reason_code: error.reason_code,
            outcome: "websocket-bad-request",
            error_label: "bad-request",
            status_code: StatusCode::BAD_REQUEST,
            message: error.message.as_str(),
        },
    )
    .await
}
