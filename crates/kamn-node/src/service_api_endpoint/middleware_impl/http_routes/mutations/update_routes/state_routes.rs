use super::super::*;

pub(super) async fn handle_post_route(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
) -> Option<Response> {
    let path = context.parsed_request.path.as_str();
    if let Some(response) = task_route_response(state, path).await {
        return Some(response);
    }
    if let Some(response) = persistence_route_response(state, path).await {
        return Some(response);
    }
    content_route_response(state, path).await
}

async fn task_route_response(state: &Arc<ServiceApiRuntimeState>, path: &str) -> Option<Response> {
    if let Some(task_id) = super::payload::task_accept_path_id(path) {
        return Some(task_transition(state, task_id, "accepted", true).await);
    }
    let task_id = super::payload::task_complete_path_id(path)?;
    Some(task_transition(state, task_id, "completed", true).await)
}

async fn persistence_route_response(
    state: &Arc<ServiceApiRuntimeState>,
    path: &str,
) -> Option<Response> {
    if let Some(escrow_id) = super::payload::escrow_release_path_id(path) {
        return Some(release_escrow(state, escrow_id).await);
    }
    let bridge_id = super::payload::bridge_forward_path_id(path)?;
    Some(forward_bridge(state, bridge_id).await)
}

async fn content_route_response(
    state: &Arc<ServiceApiRuntimeState>,
    path: &str,
) -> Option<Response> {
    if let Some(content_id) = super::payload::content_expire_path_id(path) {
        return Some(update_content(state, content_id, ContentAction::Expire).await);
    }
    let content_id = super::payload::content_tombstone_path_id(path)?;
    Some(update_content(state, content_id, ContentAction::Tombstone).await)
}

async fn task_transition(
    state: &Arc<ServiceApiRuntimeState>,
    task_id: &str,
    target: &str,
    publish: bool,
) -> Response {
    let result = state
        .message_store
        .lock()
        .await
        .transition_task(task_id, target);
    match result {
        Ok(Some(payload)) => {
            if publish {
                state
                    .websocket_events
                    .publish_task_transition_event(&payload);
            }
            contract_json(200, &payload)
        }
        Ok(None) => not_found(),
        Err(error) => persistence_error("service api task persistence failed", error),
    }
}

async fn release_escrow(state: &Arc<ServiceApiRuntimeState>, escrow_id: &str) -> Response {
    let result = state.message_store.lock().await.release_escrow(escrow_id);
    match result {
        Ok(Some(payload)) => contract_json(200, &payload),
        Ok(None) => not_found(),
        Err(error) => persistence_error("service api escrow persistence failed", error),
    }
}

enum ContentAction {
    Expire,
    Tombstone,
}

async fn update_content(
    state: &Arc<ServiceApiRuntimeState>,
    content_id: &str,
    action: ContentAction,
) -> Response {
    let result = {
        let mut store = state.message_store.lock().await;
        match action {
            ContentAction::Expire => store.expire_content(content_id),
            ContentAction::Tombstone => store.tombstone_content(content_id),
        }
    };
    match result {
        Ok(Some(payload)) => contract_json(200, &payload),
        Ok(None) => not_found(),
        Err(error) => persistence_error("service api content persistence failed", error),
    }
}

async fn forward_bridge(state: &Arc<ServiceApiRuntimeState>, bridge_id: &str) -> Response {
    let result = match resolve_bridge_forward_result(state, bridge_id).await {
        Ok(result) => result,
        Err(error) => return live_bridge_dispatch_error(error.as_str()),
    };
    match result {
        Ok(Some(payload)) => {
            state
                .websocket_events
                .publish_bridge_forwarded_event(&payload);
            contract_json(200, &payload)
        }
        Ok(None) => not_found(),
        Err(error) => persistence_error("service api bridge persistence failed", error),
    }
}

async fn resolve_bridge_forward_result(
    state: &Arc<ServiceApiRuntimeState>,
    bridge_id: &str,
) -> Result<Result<Option<ServiceApiBridgeStatusBody>, String>, String> {
    let Some(config) = state.live_solana_bridge_dispatch.as_ref() else {
        return Ok(state.message_store.lock().await.forward_bridge(bridge_id));
    };
    let evidence = crate::service_api_endpoint::live_bridge_dispatch::collect_live_bridge_forward_evidence(
        config,
        bridge_id,
    )?;
    Ok(state.message_store.lock().await.forward_bridge_with_evidence(
        bridge_id,
        evidence.target_message_id.as_str(),
        evidence.forward_tx_hash.as_str(),
    ))
}

fn live_bridge_dispatch_error(error: &str) -> Response {
    super::payload::json_error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal",
        REASON_CODE_LIVE_BRIDGE_DISPATCH_FAILED,
        format!("service api live bridge dispatch failed: {error}").as_str(),
    )
}
