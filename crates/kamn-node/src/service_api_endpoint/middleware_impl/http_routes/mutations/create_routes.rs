use super::*;

mod escrow;

type ResponseError = Box<Response>;

pub(super) async fn handle_post_route(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
) -> Option<Response> {
    match context.parsed_request.path.as_str() {
        ROUTE_CHANNELS_CREATE => Some(create_channel(state, context).await),
        ROUTE_TASKS_CREATE => Some(create_task(state, context).await),
        ROUTE_AGENTS_SEARCH => Some(search_agents(state, context).await),
        ROUTE_AGENTS_REGISTER => Some(register_agent(state, context).await),
        ROUTE_CONTENT_REGISTER => Some(register_content(state, context).await),
        ROUTE_BRIDGE_SUBMIT => Some(submit_bridge(state, context).await),
        ROUTE_ESCROW_FUND => Some(escrow::fund_escrow(state, context).await),
        _ => None,
    }
}

async fn create_channel(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
) -> Response {
    let result = state
        .message_store
        .lock()
        .await
        .create_channel(context.parsed_request.body.as_str());
    match result {
        Ok(payload) => {
            state
                .websocket_events
                .publish_channel_created_event(&payload);
            contract_json(201, &payload)
        }
        Err(error) => persistence_error("service api channel persistence failed", error),
    }
}

async fn create_task(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
) -> Response {
    let actor_did = match super::super::task_actor(context) {
        Ok(actor) => actor,
        Err(response) => return *response,
    };
    let result = {
        let mut store = state.message_store.lock().await;
        if let Err(response) =
            super::super::revalidate_transaction_authorization(&mut store, context)
        {
            return *response;
        }
        store.create_bound_task(
            actor_did.as_str(),
            context.parsed_request.body.as_str(),
            context.correlation_id.as_str(),
        )
    };
    match result {
        Ok(payload) => {
            state
                .websocket_events
                .publish_task_submitted_event(&payload);
            contract_json(201, &payload)
        }
        Err(error) => super::super::task_lifecycle_error_response(error),
    }
}

async fn search_agents(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
) -> Response {
    let search = match parse_agent_search_payload(context.parsed_request.body.as_str()) {
        Ok(payload) => payload,
        Err(error) => return bad_request(error),
    };
    let result = state
        .message_store
        .lock()
        .await
        .search_agent_profiles(&search);
    match result {
        Ok(payload) => contract_json(200, &payload),
        Err(error) => persistence_error("service api agent search persistence failed", error),
    }
}

async fn register_agent(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
) -> Response {
    let sender_did = match sender_did_from_context(context) {
        Ok(value) => value,
        Err(response) => return *response,
    };
    let registration = match registration_from_context(context) {
        Ok(value) => value,
        Err(response) => return *response,
    };
    let result = state
        .message_store
        .lock()
        .await
        .register_agent_profile(sender_did, &registration);
    registration_response(result)
}

fn sender_did_from_context(context: &ServiceApiRequestContext) -> Result<&str, ResponseError> {
    super::auth::header_value(
        &context.parsed_request.headers,
        REQUEST_AUTH_SENDER_DID_HEADER,
    )
    .ok_or_else(|| Box::new(missing_sender_did_response()))
}

fn missing_sender_did_response() -> Response {
    super::payload::json_error_response(
        StatusCode::UNAUTHORIZED,
        "unauthorized",
        REASON_CODE_AUTH_SENDER_DID_HEADER_MISSING,
        format!("missing required header: {REQUEST_AUTH_SENDER_DID_HEADER}").as_str(),
    )
}

fn registration_from_context(
    context: &ServiceApiRequestContext,
) -> Result<ServiceApiAgentRegisterRequestBody, ResponseError> {
    parse_agent_registration_payload(context.parsed_request.body.as_str())
        .map_err(|error| Box::new(bad_request(error)))
}

fn registration_response(
    result: Result<ServiceApiAgentGetBody, message_store::ServiceApiAgentRegistrationStoreError>,
) -> Response {
    match result {
        Ok(payload) => contract_json(201, &payload),
        Err(message_store::ServiceApiAgentRegistrationStoreError::Conflict(error)) => {
            registration_conflict_response(error)
        }
        Err(message_store::ServiceApiAgentRegistrationStoreError::Persistence(error)) => {
            persistence_error("service api agent registration persistence failed", error)
        }
    }
}

fn registration_conflict_response(error: impl std::fmt::Display) -> Response {
    super::payload::json_error_response(
        StatusCode::CONFLICT,
        "conflict",
        REASON_CODE_AGENT_REGISTRATION_CONFLICT,
        format!("service api agent registration rejected: {error}").as_str(),
    )
}

async fn register_content(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
) -> Response {
    let result = state
        .message_store
        .lock()
        .await
        .register_content(context.parsed_request.body.as_str());
    match result {
        Ok(payload) => contract_json(201, &payload),
        Err(error) => persistence_error("service api content persistence failed", error),
    }
}

async fn submit_bridge(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
) -> Response {
    let result = state
        .message_store
        .lock()
        .await
        .submit_bridge(context.parsed_request.body.as_str());
    match result {
        Ok(payload) => {
            state
                .websocket_events
                .publish_bridge_submitted_event(&payload);
            contract_json(202, &payload)
        }
        Err(error) => persistence_error("service api bridge persistence failed", error),
    }
}
