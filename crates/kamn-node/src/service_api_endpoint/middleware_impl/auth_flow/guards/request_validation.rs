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
    run_auth_checks(state, parsed_request, correlation_id, request_started_at).await?;
    run_policy_checks(state, parsed_request, correlation_id, request_started_at).await?;
    policy_checks::validate_websocket_requirements(
        state,
        parsed_request,
        correlation_id,
        request_started_at,
        is_websocket_route,
    )
    .await
}

async fn run_auth_checks(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
) -> Result<(), Response> {
    auth_checks::log_request_received(state, parsed_request, correlation_id, request_started_at)
        .await?;
    auth_checks::authorize_request(state, parsed_request, correlation_id, request_started_at)
        .await?;
    auth_checks::persist_auth_nonce(state, parsed_request, correlation_id, request_started_at).await
}

async fn run_policy_checks(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
) -> Result<(), Response> {
    policy_checks::enforce_scope_policy(state, parsed_request, correlation_id, request_started_at)
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

pub(super) async fn internal_response(
    state: &Arc<ServiceApiRuntimeState>,
    request_started_at: Instant,
    correlation_id: &str,
    parsed_request: &ParsedRequest,
    reason_code: &'static str,
    outcome: &str,
    error_label: &str,
    status_code: StatusCode,
    message: &str,
) -> Response {
    let error = middleware_error(
        correlation_id,
        parsed_request,
        reason_code,
        outcome,
        error_label,
        status_code,
        message,
    );
    service_api_middleware_error_response(state, request_started_at, error).await
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
