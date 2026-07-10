use super::super::*;

pub(super) async fn fund_escrow(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
) -> Response {
    let result = {
        let mut store = state.message_store.lock().await;
        if let Err(response) =
            super::super::super::revalidate_transaction_authorization(&mut store, context)
        {
            return *response;
        }
        store.fund_escrow(context.parsed_request.body.as_str())
    };
    match result {
        Ok(payload) => contract_json(200, &payload),
        Err(error) => persistence_error("service api escrow persistence failed", error),
    }
}
