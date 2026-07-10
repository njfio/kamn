use super::super::*;

mod auth_checks;
mod policy_checks;

pub(super) async fn validate_request(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
    is_websocket_route: bool,
) -> Result<(), Response> {
    verify_request_identity(state, parsed_request, correlation_id, request_started_at).await?;
    run_policy_checks(state, parsed_request, correlation_id, request_started_at).await?;
    record_request_nonce(state, parsed_request, correlation_id, request_started_at).await?;
    policy_checks::validate_websocket_requirements(
        state,
        parsed_request,
        correlation_id,
        request_started_at,
        is_websocket_route,
    )
    .await
}

async fn verify_request_identity(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
) -> Result<(), Response> {
    auth_checks::log_request_received(state, parsed_request, correlation_id, request_started_at)
        .await?;
    auth_checks::verify_request_identity(state, parsed_request, correlation_id, request_started_at)
        .await
}

async fn run_policy_checks(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
) -> Result<(), Response> {
    policy_checks::enforce_scope_policy(state, parsed_request, correlation_id, request_started_at)
        .await?;
    policy_checks::enforce_transaction_authorization(
        state,
        parsed_request,
        correlation_id,
        request_started_at,
    )
    .await?;
    policy_checks::enforce_sender_anti_spam(
        state,
        parsed_request,
        correlation_id,
        request_started_at,
    )
    .await?;
    policy_checks::enforce_ingress_rate_limit(
        state,
        parsed_request,
        correlation_id,
        request_started_at,
    )
    .await
}

async fn record_request_nonce(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
) -> Result<(), Response> {
    auth_checks::record_request_nonce(state, parsed_request, correlation_id, request_started_at)
        .await
}

pub(super) async fn internal_response(
    state: &Arc<ServiceApiRuntimeState>,
    request_started_at: Instant,
    parsed_request: &ParsedRequest,
    response: InternalResponseProjection<'_>,
) -> Response {
    let error = middleware_error(
        response.correlation_id,
        parsed_request,
        response.reason_code,
        response.outcome,
        response.error_label,
        response.status_code,
        response.message,
    );
    service_api_middleware_error_response(state, request_started_at, error).await
}

pub(super) struct InternalResponseProjection<'a> {
    pub(super) correlation_id: &'a str,
    pub(super) reason_code: &'static str,
    pub(super) outcome: &'a str,
    pub(super) error_label: &'a str,
    pub(super) status_code: StatusCode,
    pub(super) message: &'a str,
}

fn middleware_error<'a>(
    correlation_id: &'a str,
    parsed_request: &'a ParsedRequest,
    reason_code: &'static str,
    outcome: &'a str,
    error_label: &'a str,
    status_code: StatusCode,
    message: &'a str,
) -> ServiceApiMiddlewareError<'a> {
    ServiceApiMiddlewareError {
        correlation_id,
        method: parsed_request.method.as_str(),
        path: parsed_request.path.as_str(),
        status_code,
        error_label,
        reason_code,
        message,
        outcome,
    }
}
