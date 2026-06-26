use super::*;

pub(super) fn attach_request_context(
    request: &mut Request,
    parsed_request: ParsedRequest,
    correlation_id: String,
) {
    request.extensions_mut().insert(ServiceApiRequestContext {
        parsed_request,
        correlation_id,
    });
}

pub(super) struct FinalizeRequestContext<'a> {
    pub(super) method: &'a str,
    pub(super) path: &'a str,
    pub(super) correlation_id: &'a str,
    pub(super) is_websocket_route: bool,
    pub(super) request_started_at: Instant,
    pub(super) concurrency_permit: tokio::sync::OwnedSemaphorePermit,
}

pub(super) async fn finalize_request(
    state: &Arc<ServiceApiRuntimeState>,
    response: Response,
    context: FinalizeRequestContext<'_>,
) -> Response {
    let status_code = response.status().as_u16();
    let outcome = response_outcome(context.is_websocket_route, &response);
    emit_outcome(
        context.correlation_id,
        context.method,
        context.path,
        status_code,
        outcome,
    );
    record_runtime_observation(state, status_code, context.request_started_at).await;
    finish_request(state, context.concurrency_permit);
    response
}

fn response_outcome(is_websocket_route: bool, response: &Response) -> &'static str {
    response
        .extensions()
        .get::<ServiceApiRequestOutcome>()
        .map(|outcome| outcome.0)
        .unwrap_or_else(|| project_response_outcome(is_websocket_route, response))
}

fn emit_outcome(correlation_id: &str, method: &str, path: &str, status_code: u16, outcome: &str) {
    let _ = emit_service_api_request_outcome(correlation_id, method, path, status_code, outcome);
}

async fn record_runtime_observation(
    state: &Arc<ServiceApiRuntimeState>,
    status_code: u16,
    request_started_at: Instant,
) {
    super::runtime_observability::record_runtime_observation(
        state.as_ref(),
        status_code,
        request_started_at.elapsed(),
    )
    .await;
}

fn finish_request(
    state: &Arc<ServiceApiRuntimeState>,
    concurrency_permit: tokio::sync::OwnedSemaphorePermit,
) {
    state.request_budget.record_request();
    drop(concurrency_permit);
}

fn project_response_outcome(is_websocket_route: bool, response: &Response) -> &'static str {
    if is_websocket_route && response.status().is_client_error() {
        return "websocket-bad-request";
    }
    "handled"
}
