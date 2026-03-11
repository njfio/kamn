use super::*;

mod auth_flow;
mod error_response;
mod http_routes;
mod lifecycle_policy;
mod payload_parsing;
mod request_parsing;
mod websocket_routes;

use lifecycle_policy::{
    emit_service_api_request_outcome, log_service_api_event_info,
    service_api_lifecycle_rejection_policy, service_api_request_correlation_id,
};
use payload_parsing::{
    extract_canonical_recipient_did_from_payload, extract_channel_id_from_payload,
    parse_agent_registration_payload, parse_agent_search_payload, parse_relay_ingest_payload,
};

pub(super) async fn service_api_auth_middleware(
    state: State<Arc<ServiceApiRuntimeState>>,
    request: Request,
    next: Next,
) -> Response {
    auth_flow::service_api_auth_middleware(state, request, next).await
}

pub(super) async fn parse_service_api_request(
    request: Request,
    is_websocket_route: bool,
    body_limit_bytes: usize,
) -> Result<(Request, ParsedRequest), ServiceApiReasonedError> {
    request_parsing::parse_service_api_request(request, is_websocket_route, body_limit_bytes).await
}

pub(super) async fn service_api_middleware_error_response(
    state: &ServiceApiRuntimeState,
    request_started_at: Instant,
    error: ServiceApiMiddlewareError<'_>,
) -> Response {
    error_response::service_api_middleware_error_response(state, request_started_at, error).await
}

pub(super) async fn handle_service_api_http_route(
    state: State<Arc<ServiceApiRuntimeState>>,
    context: Extension<ServiceApiRequestContext>,
) -> Response {
    http_routes::handle_service_api_http_route(state, context).await
}

pub(super) async fn handle_service_api_websocket_route(
    state: State<Arc<ServiceApiRuntimeState>>,
    context: Extension<ServiceApiRequestContext>,
    upgrade: WebSocketUpgrade,
) -> Response {
    websocket_routes::handle_service_api_websocket_route(state, context, upgrade).await
}

pub(super) fn route_requires_auth(method: &str, path: &str) -> bool {
    lifecycle_policy::route_requires_auth(method, path)
}

#[cfg(test)]
pub(crate) fn project_service_api_lifecycle_rejection(
    reason_code: &str,
) -> Option<ServiceApiLifecycleRejectionProjection> {
    lifecycle_policy::project_service_api_lifecycle_rejection(reason_code)
}
