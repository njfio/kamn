use super::*;

mod task_projection;

pub(super) async fn handle_get_route(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
) -> Option<Response> {
    if let Some(response) = task_projection::handle_task_projection_query(state, context).await {
        return Some(response);
    }
    if let Some(message_id) = super::payload::message_path_id(context.parsed_request.path.as_str())
    {
        return Some(message_query(state, &context.parsed_request, message_id).await);
    }
    if let Some(channel_id) =
        super::payload::channel_messages_path_id(context.parsed_request.path.as_str())
    {
        return Some(channel_query(state, channel_id).await);
    }
    if let Some(task_id) = super::payload::task_path_id(context.parsed_request.path.as_str()) {
        return Some(task_query(state, context, task_id).await);
    }
    if let Some(content_id) = super::payload::content_path_id(context.parsed_request.path.as_str())
    {
        return Some(content_query(state, content_id).await);
    }
    if let Some(bridge_id) = super::payload::bridge_path_id(context.parsed_request.path.as_str()) {
        return Some(bridge_query(state, bridge_id).await);
    }
    agent_like_query(state, context).await
}

async fn message_query(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    message_id: &str,
) -> Response {
    let requester_did =
        super::auth::header_value(&parsed_request.headers, REQUEST_AUTH_SENDER_DID_HEADER);
    let result = {
        state
            .message_store
            .lock()
            .await
            .get_message_for_requester(message_id, requester_did)
    };
    match result {
        Ok(Some(payload)) => contract_json(200, &payload),
        Ok(None) => not_found(),
        Err(error) => persistence_error("service api message persistence failed", error),
    }
}

async fn channel_query(state: &Arc<ServiceApiRuntimeState>, channel_id: &str) -> Response {
    let result = {
        state
            .message_store
            .lock()
            .await
            .list_channel_messages(channel_id)
    };
    match result {
        Ok(payload) => contract_json(200, &payload),
        Err(error) => persistence_error("service api channel query failed", error),
    }
}

async fn task_query(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
    task_id: &str,
) -> Response {
    let result = {
        let mut store = state.message_store.lock().await;
        if let Err(response) = super::revalidate_transaction_authorization(&mut store, context) {
            return *response;
        }
        store.get_task(task_id)
    };
    match result {
        Ok(Some(payload)) => contract_json(200, &payload),
        Ok(None) => not_found(),
        Err(error) if is_task_dispatch_prerequisite_error(error.as_str()) => {
            super::payload::json_error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "service-unavailable",
                REASON_CODE_TASK_DISPATCH_PREREQUISITES_MISSING,
                error.as_str(),
            )
        }
        Err(error) => persistence_error("service api task query failed", error),
    }
}

async fn content_query(state: &Arc<ServiceApiRuntimeState>, content_id: &str) -> Response {
    let result = { state.message_store.lock().await.get_content(content_id) };
    match result {
        Ok(Some(payload)) => contract_json(200, &payload),
        Ok(None) => not_found(),
        Err(error) => persistence_error("service api content query failed", error),
    }
}

async fn bridge_query(state: &Arc<ServiceApiRuntimeState>, bridge_id: &str) -> Response {
    let result = { state.message_store.lock().await.get_bridge(bridge_id) };
    match result {
        Ok(Some(payload)) => contract_json(200, &payload),
        Ok(None) => not_found(),
        Err(error) => persistence_error("service api bridge query failed", error),
    }
}

async fn agent_like_query(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
) -> Option<Response> {
    match super::payload::canonical_agent_path_id(context.parsed_request.path.as_str()) {
        Ok(Some(agent_did)) => return Some(agent_profile_query(state, agent_did).await),
        Err(error) => return Some(invalid_agent_did(error)),
        Ok(None) => {}
    }
    match super::payload::canonical_agent_balance_path_id(context.parsed_request.path.as_str()) {
        Ok(Some(agent_did)) => Some(agent_balance_query(state, agent_did).await),
        Err(error) => Some(invalid_agent_did(error)),
        Ok(None) => None,
    }
}

async fn agent_profile_query(state: &Arc<ServiceApiRuntimeState>, agent_did: &str) -> Response {
    let result = {
        state
            .message_store
            .lock()
            .await
            .get_or_create_agent_profile(agent_did)
    };
    match result {
        Ok(payload) => contract_json(200, &payload),
        Err(error) => persistence_error("service api agent profile persistence failed", error),
    }
}

async fn agent_balance_query(state: &Arc<ServiceApiRuntimeState>, agent_did: &str) -> Response {
    let result = {
        state
            .message_store
            .lock()
            .await
            .get_or_create_agent_balance(agent_did)
    };
    match result {
        Ok(payload) => contract_json(200, &payload),
        Err(error) => persistence_error("service api agent balance persistence failed", error),
    }
}

fn invalid_agent_did(error: ServiceApiReasonedError) -> Response {
    super::payload::contract_response(super::payload::invalid_agent_did_path_endpoint_response(
        &error,
    ))
}

fn contract_json(status_code: u16, payload: &impl Serialize) -> Response {
    super::payload::contract_response(ServiceApiEndpointResponse {
        status_code,
        content_type: "application/json",
        body: super::serialize_service_api_json(payload),
    })
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

fn is_task_dispatch_prerequisite_error(error: &str) -> bool {
    error.contains("task dispatch prerequisites missing")
}
