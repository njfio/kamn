use super::*;

mod create_routes;
mod update_routes;

pub(super) async fn handle_post_route(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
) -> Option<Response> {
    if let Some(response) = create_routes::handle_post_route(state, context).await {
        return Some(response);
    }
    update_routes::handle_post_route(state, context).await
}

fn contract_json(status_code: u16, payload: &impl Serialize) -> Response {
    super::payload::contract_response(ServiceApiEndpointResponse {
        status_code,
        content_type: "application/json",
        body: super::serialize_service_api_json(payload),
    })
}

fn bad_request(error: ServiceApiReasonedError) -> Response {
    super::payload::json_error_response(
        StatusCode::BAD_REQUEST,
        "bad-request",
        error.reason_code,
        error.message.as_str(),
    )
}

fn not_found() -> Response {
    super::payload::json_error_response(
        StatusCode::NOT_FOUND,
        "not-found",
        REASON_CODE_ROUTE_NOT_FOUND,
        "not found",
    )
}

fn persistence_error(error_prefix: &str, error: impl std::fmt::Display) -> Response {
    super::payload::json_error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal",
        REASON_CODE_STATE_PERSISTENCE_FAILED,
        format!("{error_prefix}: {error}").as_str(),
    )
}
