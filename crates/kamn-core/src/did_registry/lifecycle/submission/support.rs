use super::*;
use crate::did_registry::models::DidMutationSubmissionKey;
pub(super) struct LifecycleSubmissionContext {
    pub(super) did: AgentDid,
    pub(super) nonce: u64,
    pub(super) idempotency_key: String,
    pub(super) submission_key: DidMutationSubmissionKey,
}

type LifecycleSubmissionParts = (
    DidSubmissionRetryClass,
    DidLifecycleMutationEvidence,
    DidChainSubmissionOutcome,
);

pub(super) fn lifecycle_submission_context(
    registry: &DidRegistry,
    request: &DidLifecycleMutationRequest,
) -> Result<LifecycleSubmissionContext, DidRegistryError> {
    let did = request.did.clone();
    let nonce = request.nonce;
    let idempotency_key = registry.idempotency_key_for_lifecycle_mutation(request)?;
    Ok(LifecycleSubmissionContext {
        submission_key: (did.clone(), nonce),
        did,
        nonce,
        idempotency_key,
    })
}

pub(super) fn build_submission_result(
    context: LifecycleSubmissionContext,
    retry_class: DidSubmissionRetryClass,
    outcome: DidChainSubmissionOutcome,
    evidence: DidLifecycleMutationEvidence,
) -> DidLifecycleChainSubmissionResult {
    DidLifecycleChainSubmissionResult {
        did: context.did,
        nonce: context.nonce,
        idempotency_key: context.idempotency_key,
        retry_class,
        outcome,
        evidence,
    }
}

pub(super) fn lifecycle_submission_parts<A: DidLifecycleChainAdapter>(
    registry: &mut DidRegistry,
    adapter: &mut A,
    context: &LifecycleSubmissionContext,
    request: &DidLifecycleMutationRequest,
) -> Result<LifecycleSubmissionParts, DidRegistryError> {
    let (retry_class, evidence) = lifecycle_retry_and_evidence(registry, context, request)?;
    let payload_hash = super::super::support::payload_hash_for_lifecycle_mutation(request)?;
    let outcome = lifecycle_submission_outcome(
        adapter,
        &context.did,
        request,
        context.nonce,
        &context.idempotency_key,
        payload_hash,
        retry_class,
    )?;
    Ok((retry_class, evidence, outcome))
}

fn lifecycle_retry_and_evidence(
    registry: &mut DidRegistry,
    context: &LifecycleSubmissionContext,
    request: &DidLifecycleMutationRequest,
) -> Result<(DidSubmissionRetryClass, DidLifecycleMutationEvidence), DidRegistryError> {
    let retry_class = lifecycle_retry_class(
        registry,
        &context.did,
        &context.submission_key,
        &context.idempotency_key,
    )?;
    let evidence = lifecycle_evidence(
        registry,
        &context.did,
        &context.submission_key,
        &context.idempotency_key,
        request,
        retry_class,
    )?;
    Ok((retry_class, evidence))
}

pub(super) fn lifecycle_retry_class(
    registry: &DidRegistry,
    did: &AgentDid,
    submission_key: &DidMutationSubmissionKey,
    idempotency_key: &str,
) -> Result<DidSubmissionRetryClass, DidRegistryError> {
    let retry_class = super::super::super::store::support::classify_lifecycle_retry_by_key(
        registry,
        submission_key,
        idempotency_key,
    );
    if retry_class == DidSubmissionRetryClass::ConflictNoRetry {
        return Err(conflicting_submission_key(
            registry,
            did,
            submission_key,
            idempotency_key,
        ));
    }
    Ok(retry_class)
}

pub(super) fn lifecycle_evidence(
    registry: &mut DidRegistry,
    did: &AgentDid,
    submission_key: &DidMutationSubmissionKey,
    idempotency_key: &str,
    request: &DidLifecycleMutationRequest,
    retry_class: DidSubmissionRetryClass,
) -> Result<DidLifecycleMutationEvidence, DidRegistryError> {
    match retry_class {
        DidSubmissionRetryClass::NewSubmission => {
            insert_new_lifecycle_evidence(registry, submission_key, idempotency_key, request)
        }
        DidSubmissionRetryClass::RetryableInFlight | DidSubmissionRetryClass::FinalizedNoRetry => {
            stored_lifecycle_evidence(registry, did, submission_key, idempotency_key)
        }
        DidSubmissionRetryClass::ConflictNoRetry => unreachable!("handled above"),
    }
}

fn insert_new_lifecycle_evidence(
    registry: &mut DidRegistry,
    submission_key: &DidMutationSubmissionKey,
    idempotency_key: &str,
    request: &DidLifecycleMutationRequest,
) -> Result<DidLifecycleMutationEvidence, DidRegistryError> {
    let evidence = registry.apply_lifecycle_mutation(request.clone())?;
    registry
        .lifecycle_submission_keys_by_did_nonce
        .insert(submission_key.clone(), idempotency_key.to_owned());
    registry
        .lifecycle_evidence_by_did_nonce
        .insert(submission_key.clone(), evidence.clone());
    Ok(evidence)
}

fn stored_lifecycle_evidence(
    registry: &DidRegistry,
    did: &AgentDid,
    submission_key: &DidMutationSubmissionKey,
    idempotency_key: &str,
) -> Result<DidLifecycleMutationEvidence, DidRegistryError> {
    registry
        .lifecycle_evidence_by_did_nonce
        .get(submission_key)
        .cloned()
        .ok_or_else(|| DidRegistryError::UnknownSubmissionIdempotencyKey {
            did: did.as_str().to_owned(),
            idempotency_key: idempotency_key.to_owned(),
        })
}

pub(super) fn lifecycle_submission_outcome<A: DidLifecycleChainAdapter>(
    adapter: &mut A,
    did: &AgentDid,
    request: &DidLifecycleMutationRequest,
    nonce: u64,
    idempotency_key: &str,
    payload_hash: String,
    retry_class: DidSubmissionRetryClass,
) -> Result<DidChainSubmissionOutcome, DidRegistryError> {
    if retry_class == DidSubmissionRetryClass::FinalizedNoRetry {
        return Ok(DidChainSubmissionOutcome::FinalizedNoOp);
    }
    adapter.submit_lifecycle_mutation(&DidLifecycleChainSubmissionRequest {
        did: did.clone(),
        actor_did: request.actor_did.clone(),
        nonce,
        action: request.action.label(),
        idempotency_key: idempotency_key.to_owned(),
        payload_hash,
    })
}

fn conflicting_submission_key(
    registry: &DidRegistry,
    did: &AgentDid,
    submission_key: &DidMutationSubmissionKey,
    idempotency_key: &str,
) -> DidRegistryError {
    DidRegistryError::ConflictingSubmissionIdempotencyKey {
        did: did.as_str().to_owned(),
        existing_key: registry
            .lifecycle_submission_keys_by_did_nonce
            .get(submission_key)
            .cloned()
            .unwrap_or_default(),
        provided_key: idempotency_key.to_owned(),
    }
}
