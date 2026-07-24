use super::super::*;
use super::state_routes_release::resolve_release_escrow_result;
mod bridge;

pub(super) async fn handle_post_route(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
) -> Option<Response> {
    let path = context.parsed_request.path.as_str();
    if let Some(response) = task_route_response(state, context, path).await {
        return Some(response);
    }
    if let Some(response) = persistence_route_response(state, context, path).await {
        return Some(response);
    }
    content_route_response(state, path).await
}
async fn task_route_response(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
    path: &str,
) -> Option<Response> {
    if let Some(task_id) = super::payload::task_accept_path_id(path) {
        return Some(task_transition(state, context, task_id, "task:accept", true).await);
    }
    let task_id = super::payload::task_complete_path_id(path)?;
    Some(task_transition(state, context, task_id, "task:complete", true).await)
}
async fn persistence_route_response(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
    path: &str,
) -> Option<Response> {
    if let Some(escrow_id) = super::payload::escrow_release_path_id(path) {
        return Some(release_escrow(state, context, escrow_id).await);
    }
    let bridge_id = super::payload::bridge_forward_path_id(path)?;
    Some(bridge::forward_bridge(state, bridge_id).await)
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
    context: &ServiceApiRequestContext,
    task_id: &str,
    target: &str,
    publish: bool,
) -> Response {
    let actor_did = match super::super::super::task_actor(context) {
        Ok(actor) => actor,
        Err(response) => return *response,
    };
    let result = {
        let mut store = state.message_store.lock().await;
        if let Err(response) =
            super::super::super::revalidate_transaction_authorization(&mut store, context)
        {
            return *response;
        }
        store.transition_bound_task(
            actor_did.as_str(),
            task_id,
            target,
            context.parsed_request.body.as_str(),
            context.correlation_id.as_str(),
        )
    };
    match result {
        Ok(payload) => {
            if publish {
                state
                    .websocket_events
                    .publish_task_transition_event(&payload);
            }
            contract_json(200, &payload)
        }
        Err(error) => super::super::super::task_lifecycle_error_response(error),
    }
}

async fn release_escrow(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
    escrow_id: &str,
) -> Response {
    if state.live_solana_settlement.is_none() && state.live_solana_bridge_dispatch.is_none() {
        return authorize_local_release(state, context, escrow_id).await;
    }
    let result = match resolve_release_escrow_result(state, context, escrow_id).await {
        Ok(result) => result,
        Err(response) => return *response,
    };
    match result {
        Ok(Some(payload)) => contract_json(200, &payload),
        Ok(None) => not_found(),
        Err(error) => persistence_error("service api escrow persistence failed", error),
    }
}

async fn authorize_local_release(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
    escrow_id: &str,
) -> Response {
    let actor = match super::super::super::task_actor(context) {
        Ok(actor) => actor,
        Err(response) => return *response,
    };
    let result = state.message_store.lock().await.authorize_escrow_release(
        actor.as_str(),
        escrow_id,
        context.parsed_request.body.as_str(),
        context.correlation_id.as_str(),
    );
    match result {
        Ok(payload) => contract_json(200, &payload),
        Err(error) => super::super::super::escrow_lifecycle_error_response(error),
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
