use super::*;

mod message_routes;
mod state_routes;
mod state_routes_release;

pub(super) async fn handle_post_route(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
) -> Option<Response> {
    if let Some(response) = message_routes::handle_post_route(state, context).await {
        return Some(response);
    }
    state_routes::handle_post_route(state, context).await
}
