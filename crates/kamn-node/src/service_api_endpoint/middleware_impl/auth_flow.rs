use super::*;

mod finalize;
mod guards;

struct RequestMetadata {
    method_label: String,
    path: String,
    request_started_at: Instant,
    is_websocket_route: bool,
}

struct PreparedAuthenticatedRequest {
    concurrency_permit: tokio::sync::OwnedSemaphorePermit,
    correlation_id: String,
    parsed_request: ParsedRequest,
    request: Request,
}

pub(super) async fn service_api_auth_middleware(
    State(state): State<Arc<ServiceApiRuntimeState>>,
    request: Request,
    next: Next,
) -> Response {
    let metadata = request_metadata(&request);
    let prepared_request = match load_authenticated_request(&state, request, &metadata).await {
        Ok(prepared_request) => prepared_request,
        Err(response) => return response,
    };
    finalize_authenticated_request(
        &state,
        &metadata,
        prepared_request.request,
        prepared_request.parsed_request,
        prepared_request.correlation_id,
        next,
        prepared_request.concurrency_permit,
    )
    .await
}

async fn load_authenticated_request(
    state: &Arc<ServiceApiRuntimeState>,
    request: Request,
    metadata: &RequestMetadata,
) -> Result<PreparedAuthenticatedRequest, Response> {
    let concurrency_permit = acquire_concurrency_permit(state, metadata).await?;
    tokio::task::yield_now().await;
    let (request, parsed_request, correlation_id) =
        prepare_authenticated_request(state, request, metadata).await?;
    Ok(PreparedAuthenticatedRequest {
        concurrency_permit,
        correlation_id,
        parsed_request,
        request,
    })
}

async fn finalize_authenticated_request(
    state: &Arc<ServiceApiRuntimeState>,
    metadata: &RequestMetadata,
    mut request: Request,
    parsed_request: ParsedRequest,
    correlation_id: String,
    next: Next,
    concurrency_permit: tokio::sync::OwnedSemaphorePermit,
) -> Response {
    finalize::attach_request_context(&mut request, parsed_request, correlation_id.clone());
    let response = next.run(request).await;
    finalize::finalize_request(
        state,
        response,
        finalize::FinalizeRequestContext {
            method: metadata.method_label.as_str(),
            path: metadata.path.as_str(),
            correlation_id: correlation_id.as_str(),
            is_websocket_route: metadata.is_websocket_route,
            request_started_at: metadata.request_started_at,
            concurrency_permit,
        },
    )
    .await
}

async fn prepare_authenticated_request(
    state: &Arc<ServiceApiRuntimeState>,
    request: Request,
    metadata: &RequestMetadata,
) -> Result<(Request, ParsedRequest, String), Response> {
    let (request, parsed_request) = parse_request_or_response(state, request, metadata).await?;
    let correlation_id = validate_request_or_response(state, &parsed_request, metadata).await?;
    Ok((request, parsed_request, correlation_id))
}

fn request_metadata(request: &Request) -> RequestMetadata {
    let method_label = request.method().to_string();
    let path = request.uri().path().to_owned();
    RequestMetadata {
        is_websocket_route: method_label == "GET" && path == ROUTE_EVENTS_WS,
        method_label,
        path,
        request_started_at: Instant::now(),
    }
}

async fn acquire_concurrency_permit(
    state: &Arc<ServiceApiRuntimeState>,
    metadata: &RequestMetadata,
) -> Result<tokio::sync::OwnedSemaphorePermit, Response> {
    guards::acquire_concurrency_permit(
        state,
        metadata.method_label.as_str(),
        metadata.path.as_str(),
        metadata.request_started_at,
    )
    .await
}

async fn parse_request_or_response(
    state: &Arc<ServiceApiRuntimeState>,
    request: Request,
    metadata: &RequestMetadata,
) -> Result<(Request, ParsedRequest), Response> {
    match parse_service_api_request(request, metadata.is_websocket_route, state.body_limit_bytes)
        .await
    {
        Ok(parsed_request) => Ok(parsed_request),
        Err(error) => {
            Err(guards::request_parse_error(state, metadata.request_started_at, error).await)
        }
    }
}

async fn validate_request_or_response(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    metadata: &RequestMetadata,
) -> Result<String, Response> {
    let correlation_id = service_api_request_correlation_id(parsed_request);
    guards::validate_request(
        state,
        parsed_request,
        correlation_id.as_str(),
        metadata.request_started_at,
        metadata.is_websocket_route,
    )
    .await?;
    Ok(correlation_id)
}
