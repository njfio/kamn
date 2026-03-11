use super::*;

pub(super) async fn service_api_middleware_error_response(
    state: &ServiceApiRuntimeState,
    request_started_at: Instant,
    error: ServiceApiMiddlewareError<'_>,
) -> Response {
    let response = build_error_response(&error);
    emit_error_outcome(&error);
    record_error_observation(state, request_started_at, &error).await;
    state.request_budget.record_request();
    response
}

fn build_error_response(error: &ServiceApiMiddlewareError<'_>) -> Response {
    super::payload::json_error_response(
        error.status_code,
        error.error_label,
        error.reason_code,
        error.message,
    )
}

fn emit_error_outcome(error: &ServiceApiMiddlewareError<'_>) {
    let _ = emit_service_api_request_outcome(
        error.correlation_id,
        error.method,
        error.path,
        error.status_code.as_u16(),
        error.outcome,
    );
}

async fn record_error_observation(
    state: &ServiceApiRuntimeState,
    request_started_at: Instant,
    error: &ServiceApiMiddlewareError<'_>,
) {
    super::runtime_observability::record_runtime_observation(
        state,
        error.status_code.as_u16(),
        request_started_at.elapsed(),
    )
    .await;
}
