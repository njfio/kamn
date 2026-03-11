use super::*;

mod concurrency;
mod request_validation;

pub(super) async fn acquire_concurrency_permit(
    state: &Arc<ServiceApiRuntimeState>,
    method: &str,
    path: &str,
    request_started_at: Instant,
) -> Result<tokio::sync::OwnedSemaphorePermit, Response> {
    concurrency::acquire_concurrency_permit(state, method, path, request_started_at).await
}

pub(super) async fn request_parse_error(
    state: &Arc<ServiceApiRuntimeState>,
    request_started_at: Instant,
    error: ServiceApiReasonedError,
) -> Response {
    concurrency::request_parse_error(state, request_started_at, error).await
}

pub(super) async fn validate_request(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
    is_websocket_route: bool,
) -> Result<(), Response> {
    request_validation::validate_request(
        state,
        parsed_request,
        correlation_id,
        request_started_at,
        is_websocket_route,
    )
    .await
}
