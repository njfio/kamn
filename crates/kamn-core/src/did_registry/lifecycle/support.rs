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
            let capability_fingerprint = document.metadata.capabilities.join(",");
            let verification_fingerprint = document
                .verification_method
                .iter()
                .map(|verification| {
                    format!(
                        "{}:{}:{}",
                        verification.id, verification.type_name, verification.public_key_multibase
                    )
                })
                .collect::<Vec<_>>()
                .join("|");
            let service_fingerprint = document
                .service
                .iter()
                .map(|service| {
                    format!(
                        "{}:{}:{}",
                        service.id, service.type_name, service.service_endpoint
                    )
                })
                .collect::<Vec<_>>()
                .join("|");
            Ok(format!(
                "{}:{}:{}:{}:{}",
                document.metadata.agent_type,
                document.metadata.model_family,
                capability_fingerprint,
                verification_fingerprint,
                service_fingerprint
            ))
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
