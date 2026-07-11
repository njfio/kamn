use super::super::*;

pub(super) async fn fund_escrow(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
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
        store.fund_bound_escrow(actor_did.as_str(), context.parsed_request.body.as_str())
    };
    match result {
        Ok(payload) => contract_json(200, &payload),
        Err(error) => super::super::super::escrow_lifecycle_error_response(error),
    }
}
