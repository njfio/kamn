use super::*;
use crate::{did_registry::models::DidRegistryRecord, DidDocument};

impl DidRegistry {
    /// Applies lifecycle mutation with nonce and actor authorization checks.
    pub fn apply_lifecycle_mutation(
        &mut self,
        request: DidLifecycleMutationRequest,
    ) -> Result<DidLifecycleMutationEvidence, DidRegistryError> {
        let did = request.did.clone();
        ensure_nonce_is_fresh(self, &did, request.nonce)?;
        let from_revoked = current_revocation_state(self, &did)?;
        support::authorize_mutation_actor(self, &did, &request.actor_did)?;
        let mutation = request.action.clone();
        let action = mutation.label();
        apply_mutation_action(self, &did, action, from_revoked, mutation)?;
        self.last_mutation_nonce_by_did
            .insert(did.clone(), request.nonce);
        Ok(build_mutation_evidence(
            &did,
            request,
            action,
            from_revoked,
            current_revocation_state(self, &did)?,
        ))
    }
}

fn ensure_nonce_is_fresh(
    registry: &DidRegistry,
    did: &AgentDid,
    nonce: u64,
) -> Result<(), DidRegistryError> {
    if nonce == 0 {
        return Err(DidRegistryError::InvalidMutationNonce {
            did: did.as_str().to_owned(),
            nonce,
        });
    }
    if let Some(last_nonce) = registry.last_mutation_nonce_by_did.get(did) {
        if nonce <= *last_nonce {
            return Err(DidRegistryError::ReplayedMutationNonce {
                did: did.as_str().to_owned(),
                last_nonce: *last_nonce,
                found: nonce,
            });
        }
    }
    Ok(())
}

fn current_revocation_state(
    registry: &DidRegistry,
    did: &AgentDid,
) -> Result<bool, DidRegistryError> {
    registry
        .records
        .get(did)
        .map(|record| record.revoked)
        .ok_or_else(|| DidRegistryError::NotFound(did.as_str().to_owned()))
}

fn apply_mutation_action(
    registry: &mut DidRegistry,
    did: &AgentDid,
    action: &'static str,
    from_revoked: bool,
    mutation: DidLifecycleMutationAction,
) -> Result<(), DidRegistryError> {
    match mutation {
        DidLifecycleMutationAction::Rotate { document } => {
            write_document_mutation(registry, did, action, from_revoked, false, document)
        }
        DidLifecycleMutationAction::Revoke => {
            validate_transition(did, action, from_revoked, false)?;
            required_record_mut(registry, did)?.revoked = true;
            Ok(())
        }
        DidLifecycleMutationAction::Recover { document } => {
            write_document_mutation(registry, did, action, from_revoked, true, document)
        }
    }
}

fn write_document_mutation(
    registry: &mut DidRegistry,
    did: &AgentDid,
    action: &'static str,
    from_revoked: bool,
    expected_revoked: bool,
    document: DidDocument,
) -> Result<(), DidRegistryError> {
    validate_transition(did, action, from_revoked, expected_revoked)?;
    super::super::store::support::validate_document_did(did, &document)?;
    let record = required_record_mut(registry, did)?;
    record.document = document;
    record.revoked = false;
    Ok(())
}

fn build_mutation_evidence(
    did: &AgentDid,
    request: DidLifecycleMutationRequest,
    action: &'static str,
    from_revoked: bool,
    to_revoked: bool,
) -> DidLifecycleMutationEvidence {
    DidLifecycleMutationEvidence {
        did: did.as_str().to_owned(),
        actor_did: request.actor_did,
        nonce: request.nonce,
        action,
        from_revoked,
        to_revoked,
        reason_code: "did_lifecycle_mutation_allowed",
    }
}

fn required_record_mut<'a>(
    registry: &'a mut DidRegistry,
    did: &AgentDid,
) -> Result<&'a mut DidRegistryRecord, DidRegistryError> {
    registry
        .records
        .get_mut(did)
        .ok_or_else(|| DidRegistryError::NotFound(did.as_str().to_owned()))
}

fn validate_transition(
    did: &AgentDid,
    action: &'static str,
    from_revoked: bool,
    expected_revoked: bool,
) -> Result<(), DidRegistryError> {
    if from_revoked != expected_revoked {
        return Err(DidRegistryError::InvalidLifecycleMutationTransition {
            did: did.as_str().to_owned(),
            action,
            from_revoked,
        });
    }
    Ok(())
}
