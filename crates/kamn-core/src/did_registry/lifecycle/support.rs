use super::*;

pub(crate) fn authorize_mutation_actor(
    registry: &DidRegistry,
    did: &AgentDid,
    actor_did: &str,
) -> Result<(), DidRegistryError> {
    let record = registry
        .records
        .get(did)
        .ok_or_else(|| DidRegistryError::NotFound(did.as_str().to_owned()))?;
    let required_actor = record
        .document
        .metadata
        .operator
        .clone()
        .unwrap_or_else(|| did.as_str().to_owned());
    if actor_did != required_actor {
        return Err(DidRegistryError::UnauthorizedMutationActor {
            did: did.as_str().to_owned(),
            actor_did: actor_did.to_owned(),
            required_actor,
        });
    }
    Ok(())
}

pub(crate) fn lifecycle_action_fingerprint(
    did: &AgentDid,
    action: &DidLifecycleMutationAction,
) -> Result<String, DidRegistryError> {
    match action {
        DidLifecycleMutationAction::Rotate { document }
        | DidLifecycleMutationAction::Recover { document } => {
            super::super::store::support::validate_document_did(did, document)?;
            Ok(super::super::store::support::document_fingerprint(document))
        }
        DidLifecycleMutationAction::Revoke => Ok("revoke".to_owned()),
    }
}

pub(crate) fn payload_hash_for_lifecycle_mutation(
    request: &DidLifecycleMutationRequest,
) -> Result<String, DidRegistryError> {
    let fingerprint = lifecycle_action_fingerprint(&request.did, &request.action)?;
    Ok(format!(
        "did-lifecycle-payload:{}:{}:{}:{}:{}",
        request.did.as_str(),
        request.actor_did,
        request.nonce,
        request.action.label(),
        fingerprint
    ))
}
