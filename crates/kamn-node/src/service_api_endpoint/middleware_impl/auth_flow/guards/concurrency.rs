use super::super::*;

pub(super) async fn acquire_concurrency_permit(
    state: &Arc<ServiceApiRuntimeState>,
    method: &str,
    path: &str,
    request_started_at: Instant,
) -> Result<tokio::sync::OwnedSemaphorePermit, Response> {
    match state.concurrency_limiter.clone().try_acquire_owned() {
        Ok(permit) => Ok(permit),
        Err(_) => Err(concurrency_limit_response(state, method, path, request_started_at).await),
    }
}

pub(super) async fn request_parse_error(
    state: &Arc<ServiceApiRuntimeState>,
    request_started_at: Instant,
    error: ServiceApiReasonedError,
) -> Response {
    let correlation_id = format!(
        "service-api:parse-error:{:016x}",
        super::super::deterministic_body_tag(error.message.as_bytes())
    );
    service_api_middleware_error_response(
        state,
        request_started_at,
        ServiceApiMiddlewareError {
            correlation_id: correlation_id.as_str(),
            method: "unknown",
            path: "unknown",
            status_code: StatusCode::BAD_REQUEST,
            error_label: "bad-request",
            reason_code: error.reason_code,
            message: error.message.as_str(),
            outcome: "bad-request",
        },
    )
    .await
}

async fn concurrency_limit_response(
    state: &Arc<ServiceApiRuntimeState>,
    method: &str,
    path: &str,
    request_started_at: Instant,
) -> Response {
    let projection = concurrency_projection();
    let correlation_id = concurrency_correlation_id(method, path);
    service_api_middleware_error_response(
        state,
        request_started_at,
        ServiceApiMiddlewareError {
            correlation_id: correlation_id.as_str(),
            method,
            path,
            status_code: projection.status_code,
            error_label: projection.error_label,
            reason_code: projection.reason_code,
            message: projection.default_message,
            outcome: projection.outcome,
        },
    )
    .await
}

fn concurrency_projection() -> ServiceApiLifecycleRejectionPolicy {
    service_api_lifecycle_rejection_policy(REASON_CODE_INGRESS_CONCURRENCY_LIMIT_EXCEEDED)
        .unwrap_or(ServiceApiLifecycleRejectionPolicy {
            rejection_class: LIFECYCLE_REJECTION_CLASS_ASYNC_LIMITER,
            reason_code: REASON_CODE_INGRESS_CONCURRENCY_LIMIT_EXCEEDED,
            status_code: StatusCode::TOO_MANY_REQUESTS,
            error_label: "too-many-requests",
            outcome: "concurrency-limit",
            default_message: "ingress concurrency limit exceeded",
        })
}

fn concurrency_correlation_id(method: &str, path: &str) -> String {
    format!(
        "service-api:{}:{}:concurrency-limit",
        method.to_ascii_lowercase(),
        path
    )
}
