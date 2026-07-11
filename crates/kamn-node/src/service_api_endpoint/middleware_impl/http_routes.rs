use super::*;

mod mutations;
mod queries;

fn revalidate_transaction_authorization(
    store: &mut ServiceApiMessageStore,
    context: &ServiceApiRequestContext,
) -> Result<(), Box<Response>> {
    let target = super::auth::resolve_transaction_authorization_target(&context.parsed_request)
        .map_err(|error| Box::new(forbidden(error.reason_code, error.message.as_str())))?;
    let Some(target) = target else {
        return Ok(());
    };
    let request = message_store::ServiceApiAuthorizationRequest {
        correlation_id: "handler-revalidation",
        actor_did: target.actor_did.as_str(),
        resource: target.resource.as_str(),
        action: target.action,
        role: target.role,
    };
    match store.revalidate_transaction_action(request) {
        Ok(decision) if decision.allowed => Ok(()),
        Ok(decision) => Err(Box::new(forbidden(
            decision.reason_code,
            "transaction authorization changed before handler execution",
        ))),
        Err(error) => Err(Box::new(persistence_error(error.as_str()))),
    }
}

fn forbidden(reason_code: &'static str, message: &str) -> Response {
    super::payload::json_error_response(StatusCode::FORBIDDEN, "forbidden", reason_code, message)
}

fn persistence_error(error: &str) -> Response {
    super::payload::json_error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal",
        REASON_CODE_STATE_PERSISTENCE_FAILED,
        format!("service api authorization revalidation failed: {error}").as_str(),
    )
}

fn task_actor(context: &ServiceApiRequestContext) -> Result<String, Box<Response>> {
    let target = super::auth::resolve_transaction_authorization_target(&context.parsed_request)
        .map_err(|error| Box::new(forbidden(error.reason_code, error.message.as_str())))?;
    target
        .map(|target| target.actor_did)
        .ok_or_else(|| Box::new(persistence_error("task authorization target is missing")))
}

fn task_lifecycle_error_response(error: message_store::TaskLifecycleError) -> Response {
    use message_store::TaskLifecycleError::*;
    match error {
        BadRequest(code, message) => task_error(StatusCode::BAD_REQUEST, code, message),
        Forbidden(code, message) => task_error(StatusCode::FORBIDDEN, code, message),
        Conflict(code, message) => task_error(StatusCode::CONFLICT, code, message),
        NotFound => super::payload::json_error_response(
            StatusCode::NOT_FOUND,
            "not-found",
            "service_api_route_not_found",
            "service api task not found",
        ),
        Persistence(message) => persistence_error(message.as_str()),
    }
}

fn escrow_lifecycle_error_response(error: message_store::EscrowLifecycleError) -> Response {
    use message_store::EscrowLifecycleError::*;
    match error {
        BadRequest(code, message) => task_error(StatusCode::BAD_REQUEST, code, message),
        Forbidden(code, message) => task_error(StatusCode::FORBIDDEN, code, message),
        Conflict(code, message) => task_error(StatusCode::CONFLICT, code, message),
        NotFound => super::payload::json_error_response(
            StatusCode::NOT_FOUND,
            "not-found",
            "service_api_route_not_found",
            "service api escrow not found",
        ),
        Persistence(message) => persistence_error(message.as_str()),
    }
}

fn task_error(status: StatusCode, code: &'static str, message: String) -> Response {
    super::payload::json_error_response(status, "task", code, message.as_str())
}

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
