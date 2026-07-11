use super::*;

pub(super) async fn handle_task_projection_query(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
) -> Option<Response> {
    let path = context.parsed_request.path.as_str();
    if let Some(task_id) = super::super::payload::task_participant_view_path_id(path) {
        return Some(participant_projection(state, context, task_id).await);
    }
    let task_id = super::super::payload::task_verifier_view_path_id(path)?;
    Some(verifier_projection(state, context, task_id).await)
}

async fn participant_projection(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
    task_id: &str,
) -> Response {
    let requester = requester_did(context);
    let result = state
        .message_store
        .lock()
        .await
        .participant_task_projection(task_id, requester);
    projection_response(result)
}

async fn verifier_projection(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
    task_id: &str,
) -> Response {
    let requester = requester_did(context);
    let result = state
        .message_store
        .lock()
        .await
        .verifier_task_projection(task_id, requester);
    projection_response(result)
}

fn requester_did(context: &ServiceApiRequestContext) -> &str {
    super::super::auth::header_value(
        &context.parsed_request.headers,
        REQUEST_AUTH_SENDER_DID_HEADER,
    )
    .unwrap_or_default()
}

fn projection_response<T: Serialize>(
    result: Result<Option<T>, message_store::TaskProjectionError>,
) -> Response {
    match result {
        Ok(Some(payload)) => super::contract_json(200, &payload),
        Ok(None) => super::not_found(),
        Err(error) => projection_contract_error(error),
    }
}

fn projection_contract_error(error: message_store::TaskProjectionError) -> Response {
    use message_store::TaskProjectionError::*;
    let (status, code, message) = match error {
        Unregistered => (
            StatusCode::FORBIDDEN,
            "AGENT_NOT_REGISTERED",
            "agent not registered",
        ),
        Forbidden => (
            StatusCode::FORBIDDEN,
            "TASK_PARTICIPANT_VIEW_FORBIDDEN",
            "not a participant",
        ),
        EscrowBindingMissing => (
            StatusCode::CONFLICT,
            "TASK_ESCROW_BINDING_MISSING",
            "escrow missing",
        ),
        Inconsistent => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "TRANSACTION_PROJECTION_INCONSISTENT",
            "projection inconsistent",
        ),
        Persistence(error) => return super::persistence_error("task projection failed", error),
    };
    super::super::payload::json_error_response(status, "task-projection", code, message)
}
