use super::*;

pub(super) fn persist(
    store: &mut ServiceApiMessageStore,
    context: &ServiceApiRequestContext,
    escrow_id: &str,
) -> Result<(), Box<Response>> {
    let actor = super::super::super::super::super::task_actor(context)?;
    store
        .authorize_escrow_release(
            actor.as_str(),
            escrow_id,
            context.parsed_request.body.as_str(),
            context.correlation_id.as_str(),
        )
        .map(|_| ())
        .map_err(|error| {
            Box::new(super::super::super::super::super::escrow_lifecycle_error_response(error))
        })
}
