use super::*;

pub(crate) fn classify_retry_by_key(
    registry: &DidRegistry,
    did: &AgentDid,
    idempotency_key: &str,
) -> DidSubmissionRetryClass {
    let Some(existing_key) = registry.submission_keys_by_did.get(did) else {
        return DidSubmissionRetryClass::NewSubmission;
    };
    if existing_key != idempotency_key {
        return DidSubmissionRetryClass::ConflictNoRetry;
    }
    if registry.finality_by_did.contains_key(did) {
        return DidSubmissionRetryClass::FinalizedNoRetry;
    }
    DidSubmissionRetryClass::RetryableInFlight
}

pub(crate) fn classify_lifecycle_retry_by_key(
    registry: &DidRegistry,
    key: &DidMutationSubmissionKey,
    idempotency_key: &str,
) -> DidSubmissionRetryClass {
    let Some(existing_key) = registry.lifecycle_submission_keys_by_did_nonce.get(key) else {
        return DidSubmissionRetryClass::NewSubmission;
    };
    if existing_key != idempotency_key {
        return DidSubmissionRetryClass::ConflictNoRetry;
    }
    if registry.lifecycle_finality_by_did_nonce.contains_key(key) {
        return DidSubmissionRetryClass::FinalizedNoRetry;
    }
    DidSubmissionRetryClass::RetryableInFlight
}

pub(crate) fn validate_document_did(
    did: &AgentDid,
    document: &DidDocument,
) -> Result<(), DidRegistryError> {
    if document.id != did.as_str() {
        return Err(DidRegistryError::DocumentDidMismatch {
            expected: did.as_str().to_owned(),
            actual: document.id.clone(),
        });
    }
    Ok(())
}
