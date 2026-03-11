use super::*;
use crate::did_registry::{
    DidChainSubmissionOutcome, DidChainSubmissionRequest, DidChainSubmissionResult,
};

impl DidRegistry {
    /// Computes deterministic idempotency key for register request.
    pub fn idempotency_key_for_register(
        &self,
        did: &AgentDid,
        document: &DidDocument,
    ) -> Result<String, DidRegistryError> {
        support::validate_document_did(did, document)?;
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
            "did-register:{}:{}:{}:{}:{}:{}",
            did.as_str(),
            document.metadata.agent_type,
            document.metadata.model_family,
            capability_fingerprint,
            verification_fingerprint,
            service_fingerprint
        ))
    }

    /// Classifies retry posture for register operation.
    pub fn classify_register_retry(
        &self,
        did: &AgentDid,
        document: &DidDocument,
    ) -> Result<DidSubmissionRetryClass, DidRegistryError> {
        let idempotency_key = self.idempotency_key_for_register(did, document)?;
        Ok(support::classify_retry_by_key(self, did, &idempotency_key))
    }

    /// Registers DID with built-in retry classification guard.
    pub fn register_with_retry_guard(
        &mut self,
        did: AgentDid,
        document: DidDocument,
    ) -> Result<DidSubmissionRetryClass, DidRegistryError> {
        let idempotency_key = self.idempotency_key_for_register(&did, &document)?;
        match support::classify_retry_by_key(self, &did, &idempotency_key) {
            DidSubmissionRetryClass::NewSubmission => {
                self.register(did.clone(), document)?;
                self.submission_keys_by_did.insert(did, idempotency_key);
                Ok(DidSubmissionRetryClass::NewSubmission)
            }
            DidSubmissionRetryClass::RetryableInFlight => {
                Ok(DidSubmissionRetryClass::RetryableInFlight)
            }
            DidSubmissionRetryClass::FinalizedNoRetry => {
                Ok(DidSubmissionRetryClass::FinalizedNoRetry)
            }
            DidSubmissionRetryClass::ConflictNoRetry => {
                let existing_key = self
                    .submission_keys_by_did
                    .get(&did)
                    .cloned()
                    .unwrap_or_default();
                Err(DidRegistryError::ConflictingSubmissionIdempotencyKey {
                    did: did.as_str().to_owned(),
                    existing_key,
                    provided_key: idempotency_key,
                })
            }
        }
    }

    /// Executes register flow through chain adapter with retry guard.
    pub fn submit_registration_via_chain_adapter<A: DidRegistrationChainAdapter>(
        &mut self,
        adapter: &mut A,
        did: AgentDid,
        document: DidDocument,
    ) -> Result<DidChainSubmissionResult, DidRegistryError> {
        let idempotency_key = self.idempotency_key_for_register(&did, &document)?;
        let retry_class = self.register_with_retry_guard(did.clone(), document.clone())?;
        let outcome = if retry_class == DidSubmissionRetryClass::FinalizedNoRetry {
            DidChainSubmissionOutcome::FinalizedNoOp
        } else {
            let request = DidChainSubmissionRequest {
                did: did.clone(),
                idempotency_key: idempotency_key.clone(),
                document,
            };
            adapter.submit_registration(&request)?
        };
        Ok(DidChainSubmissionResult {
            did,
            idempotency_key,
            retry_class,
            outcome,
        })
    }
}
