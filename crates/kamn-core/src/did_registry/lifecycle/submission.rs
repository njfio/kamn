use self::submission_support::*;
use super::*;

#[path = "submission/support.rs"]
mod submission_support;

impl DidRegistry {
    /// Computes deterministic idempotency key for lifecycle mutation request.
    pub fn idempotency_key_for_lifecycle_mutation(
        &self,
        request: &DidLifecycleMutationRequest,
    ) -> Result<String, DidRegistryError> {
        if request.nonce == 0 {
            return Err(DidRegistryError::InvalidMutationNonce {
                did: request.did.as_str().to_owned(),
                nonce: request.nonce,
            });
        }
        let action_fingerprint =
            super::support::lifecycle_action_fingerprint(&request.did, &request.action)?;
        Ok(format!(
            "did-lifecycle:{}:{}:{}:{}:{}",
            request.did.as_str(),
            request.actor_did,
            request.nonce,
            request.action.label(),
            action_fingerprint
        ))
    }

    /// Submits lifecycle mutation through chain adapter with deterministic retry classification.
    pub fn submit_lifecycle_mutation_via_chain_adapter<A: DidLifecycleChainAdapter>(
        &mut self,
        adapter: &mut A,
        request: DidLifecycleMutationRequest,
    ) -> Result<DidLifecycleChainSubmissionResult, DidRegistryError> {
        let context = lifecycle_submission_context(self, &request)?;
        let (retry_class, evidence, outcome) =
            lifecycle_submission_parts(self, adapter, &context, &request)?;
        Ok(build_submission_result(
            context,
            retry_class,
            outcome,
            evidence,
        ))
    }
}
