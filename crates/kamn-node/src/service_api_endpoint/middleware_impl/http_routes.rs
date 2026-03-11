use super::*;

mod mutations;
mod queries;

pub(super) async fn handle_service_api_http_route(
    State(state): State<Arc<ServiceApiRuntimeState>>,
    Extension(context): Extension<ServiceApiRequestContext>,
) -> Response {
    let _ = context.correlation_id.as_str();
    if context.parsed_request.method == "POST" {
        if let Some(response) = mutations::handle_post_route(&state, &context).await {
            return response;
        }
    }
    if context.parsed_request.method == "GET" {
        if let Some(response) = queries::handle_get_route(&state, &context).await {
            return response;
        }
    }
    render_snapshot_response(&state, &context).await
}

async fn render_snapshot_response(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
) -> Response {
    let snapshot =
        super::runtime_observability::snapshot_with_runtime_observability(state.as_ref()).await;
    let rendered = super::render_service_api_endpoint_response(
        &snapshot,
        context.parsed_request.method.as_str(),
        context.parsed_request.path.as_str(),
        context.parsed_request.body.as_str(),
    );
    super::payload::contract_response(rendered)
}
