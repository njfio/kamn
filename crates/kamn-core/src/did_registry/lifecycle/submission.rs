use super::*;

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
            support::lifecycle_action_fingerprint(&request.did, &request.action)?;
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
        let did = request.did.clone();
        let nonce = request.nonce;
        let idempotency_key = self.idempotency_key_for_lifecycle_mutation(&request)?;
        let submission_key = (did.clone(), nonce);
        let retry_class = super::super::store::support::classify_lifecycle_retry_by_key(
            self,
            &submission_key,
            idempotency_key.as_str(),
        );
        if retry_class == DidSubmissionRetryClass::ConflictNoRetry {
            let existing_key = self
                .lifecycle_submission_keys_by_did_nonce
                .get(&submission_key)
                .cloned()
                .unwrap_or_default();
            return Err(DidRegistryError::ConflictingSubmissionIdempotencyKey {
                did: did.as_str().to_owned(),
                existing_key,
                provided_key: idempotency_key,
            });
        }
        let evidence = match retry_class {
            DidSubmissionRetryClass::NewSubmission => {
                let value = self.apply_lifecycle_mutation(request.clone())?;
                self.lifecycle_submission_keys_by_did_nonce
                    .insert(submission_key.clone(), idempotency_key.clone());
                self.lifecycle_evidence_by_did_nonce
                    .insert(submission_key.clone(), value.clone());
                value
            }
            DidSubmissionRetryClass::RetryableInFlight
            | DidSubmissionRetryClass::FinalizedNoRetry => self
                .lifecycle_evidence_by_did_nonce
                .get(&submission_key)
                .cloned()
                .ok_or_else(|| DidRegistryError::UnknownSubmissionIdempotencyKey {
                    did: did.as_str().to_owned(),
                    idempotency_key: idempotency_key.clone(),
                })?,
            DidSubmissionRetryClass::ConflictNoRetry => unreachable!("handled above"),
        };
        let payload_hash = support::payload_hash_for_lifecycle_mutation(&request)?;
        let outcome = if retry_class == DidSubmissionRetryClass::FinalizedNoRetry {
            DidChainSubmissionOutcome::FinalizedNoOp
        } else {
            adapter.submit_lifecycle_mutation(&DidLifecycleChainSubmissionRequest {
                did: did.clone(),
                actor_did: request.actor_did,
                nonce,
                action: request.action.label(),
                idempotency_key: idempotency_key.clone(),
                payload_hash,
            })?
        };
        Ok(DidLifecycleChainSubmissionResult {
            did,
            nonce,
            idempotency_key,
            retry_class,
            outcome,
            evidence,
        })
    }
}
