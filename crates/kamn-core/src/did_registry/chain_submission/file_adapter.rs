use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
/// File-backed chain adapter with deterministic idempotency persistence.
pub struct FileDidRegistrationChainAdapter {
    provider: String,
    path: PathBuf,
    receipts_by_key: SubmissionReceiptIndex,
    rejected_reasons_by_key: SubmissionRejectIndex,
}

impl FileDidRegistrationChainAdapter {
    /// Creates a file-backed adapter from persisted state.
    pub fn new(path: PathBuf, provider: &str) -> Result<Self, DidRegistryError> {
        if path.as_os_str().is_empty() {
            return Err(DidRegistryError::PersistenceInvalidPayload(
                "did chain adapter path cannot be empty".to_owned(),
            ));
        }
        if provider.trim().is_empty() {
            return Err(DidRegistryError::PersistenceInvalidPayload(
                "did chain adapter provider cannot be empty".to_owned(),
            ));
        }
        let (receipts_by_key, rejected_reasons_by_key) = read_did_chain_adapter_file(&path)?;
        Ok(Self {
            provider: provider.to_owned(),
            path,
            receipts_by_key,
            rejected_reasons_by_key,
        })
    }

    /// Configures an idempotency key to return a rejected outcome.
    pub fn reject_idempotency_key(
        &mut self,
        idempotency_key: &str,
        reason: &str,
    ) -> Result<(), DidRegistryError> {
        self.rejected_reasons_by_key
            .insert(idempotency_key.to_owned(), reason.to_owned());
        persist_did_chain_adapter_file(
            &self.path,
            &self.receipts_by_key,
            &self.rejected_reasons_by_key,
        )
    }

    fn submit_persisted(
        &mut self,
        idempotency_key: &str,
        transaction_id: String,
    ) -> Result<DidChainSubmissionOutcome, DidRegistryError> {
        if let Some(outcome) = self.existing_outcome(idempotency_key) {
            return Ok(outcome);
        }
        let receipt = DidChainSubmissionReceipt {
            provider: self.provider.clone(),
            transaction_id,
        };
        self.receipts_by_key
            .insert(idempotency_key.to_owned(), receipt.clone());
        self.persist_state()?;
        Ok(DidChainSubmissionOutcome::Submitted(receipt))
    }

    fn existing_outcome(&self, idempotency_key: &str) -> Option<DidChainSubmissionOutcome> {
        if let Some(reason) = self.rejected_reasons_by_key.get(idempotency_key) {
            return Some(DidChainSubmissionOutcome::Rejected {
                reason: reason.clone(),
            });
        }
        self.receipts_by_key
            .get(idempotency_key)
            .cloned()
            .map(DidChainSubmissionOutcome::Duplicate)
    }

    fn persist_state(&self) -> Result<(), DidRegistryError> {
        persist_did_chain_adapter_file(
            &self.path,
            &self.receipts_by_key,
            &self.rejected_reasons_by_key,
        )
    }
}

impl DidRegistrationChainAdapter for FileDidRegistrationChainAdapter {
    fn submit_registration(
        &mut self,
        request: &DidChainSubmissionRequest,
    ) -> Result<DidChainSubmissionOutcome, DidRegistryError> {
        self.submit_persisted(
            &request.idempotency_key,
            format!(
                "did-tx:{}:{}",
                request.did.method_specific_id(),
                request.idempotency_key.len()
            ),
        )
    }
}

impl DidLifecycleChainAdapter for FileDidRegistrationChainAdapter {
    fn submit_lifecycle_mutation(
        &mut self,
        request: &DidLifecycleChainSubmissionRequest,
    ) -> Result<DidChainSubmissionOutcome, DidRegistryError> {
        self.submit_persisted(
            &request.idempotency_key,
            format!(
                "did-lifecycle-tx:{}:{}",
                request.did.method_specific_id(),
                request.nonce
            ),
        )
    }
}
