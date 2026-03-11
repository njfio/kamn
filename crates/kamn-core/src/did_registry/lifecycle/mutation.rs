use super::*;
use crate::did_registry::models::DidRegistryRecord;

impl DidRegistry {
    /// Applies lifecycle mutation with nonce and actor authorization checks.
    pub fn apply_lifecycle_mutation(
        &mut self,
        request: DidLifecycleMutationRequest,
    ) -> Result<DidLifecycleMutationEvidence, DidRegistryError> {
        let did = request.did;
        let did_id = did.as_str().to_owned();
        if request.nonce == 0 {
            return Err(DidRegistryError::InvalidMutationNonce {
                did: did_id,
                nonce: request.nonce,
            });
        }
        if let Some(last_nonce) = self.last_mutation_nonce_by_did.get(&did) {
            if request.nonce <= *last_nonce {
                return Err(DidRegistryError::ReplayedMutationNonce {
                    did: did.as_str().to_owned(),
                    last_nonce: *last_nonce,
                    found: request.nonce,
                });
            }
        }
        let from_revoked = self
            .records
            .get(&did)
            .map(|record| record.revoked)
            .ok_or_else(|| DidRegistryError::NotFound(did.as_str().to_owned()))?;
        support::authorize_mutation_actor(self, &did, &request.actor_did)?;
        let action = request.action.label();
        match request.action {
            DidLifecycleMutationAction::Rotate { document } => {
                validate_transition(&did, action, from_revoked, false)?;
                super::super::store::support::validate_document_did(&did, &document)?;
                let record = required_record_mut(self, &did)?;
                record.document = document;
            }
            DidLifecycleMutationAction::Revoke => {
                validate_transition(&did, action, from_revoked, false)?;
                required_record_mut(self, &did)?.revoked = true;
            }
            DidLifecycleMutationAction::Recover { document } => {
                validate_transition(&did, action, from_revoked, true)?;
                super::super::store::support::validate_document_did(&did, &document)?;
                let record = required_record_mut(self, &did)?;
                record.document = document;
                record.revoked = false;
            }
        }
        self.last_mutation_nonce_by_did
            .insert(did.clone(), request.nonce);
        let to_revoked = self
            .records
            .get(&did)
            .map(|record| record.revoked)
            .unwrap_or(false);
        Ok(DidLifecycleMutationEvidence {
            did: did.as_str().to_owned(),
            actor_did: request.actor_did,
            nonce: request.nonce,
            action,
            from_revoked,
            to_revoked,
            reason_code: "did_lifecycle_mutation_allowed",
        })
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
