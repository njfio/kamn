use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// In-memory chain adapter used for deterministic tests.
pub struct InMemoryDidRegistrationChainAdapter {
    provider: String,
    receipts_by_key: HashMap<String, DidChainSubmissionReceipt>,
    rejected_reasons_by_key: HashMap<String, String>,
}

impl InMemoryDidRegistrationChainAdapter {
    /// Creates an in-memory adapter with provider label.
    pub fn new(provider: &str) -> Self {
        Self {
            provider: provider.to_owned(),
            receipts_by_key: HashMap::new(),
            rejected_reasons_by_key: HashMap::new(),
        }
    }

    /// Configures an idempotency key to return a rejected outcome.
    pub fn reject_idempotency_key(&mut self, idempotency_key: &str, reason: &str) {
        self.rejected_reasons_by_key
            .insert(idempotency_key.to_owned(), reason.to_owned());
    }
}

impl DidRegistrationChainAdapter for InMemoryDidRegistrationChainAdapter {
    fn submit_registration(
        &mut self,
        request: &DidChainSubmissionRequest,
    ) -> Result<DidChainSubmissionOutcome, DidRegistryError> {
        submit_outcome(
            &self.provider,
            &mut self.receipts_by_key,
            &self.rejected_reasons_by_key,
            &request.idempotency_key,
            format!(
                "did-tx:{}:{}",
                request.did.method_specific_id(),
                request.idempotency_key.len()
            ),
        )
    }
}

impl DidLifecycleChainAdapter for InMemoryDidRegistrationChainAdapter {
    fn submit_lifecycle_mutation(
        &mut self,
        request: &DidLifecycleChainSubmissionRequest,
    ) -> Result<DidChainSubmissionOutcome, DidRegistryError> {
        submit_outcome(
            &self.provider,
            &mut self.receipts_by_key,
            &self.rejected_reasons_by_key,
            &request.idempotency_key,
            format!(
                "did-lifecycle-tx:{}:{}",
                request.did.method_specific_id(),
                request.nonce
            ),
        )
    }
}

fn submit_outcome(
    provider: &str,
    receipts_by_key: &mut HashMap<String, DidChainSubmissionReceipt>,
    rejected_reasons_by_key: &HashMap<String, String>,
    idempotency_key: &str,
    transaction_id: String,
) -> Result<DidChainSubmissionOutcome, DidRegistryError> {
    if let Some(reason) = rejected_reasons_by_key.get(idempotency_key) {
        return Ok(DidChainSubmissionOutcome::Rejected {
            reason: reason.clone(),
        });
    }
    if let Some(existing) = receipts_by_key.get(idempotency_key) {
        return Ok(DidChainSubmissionOutcome::Duplicate(existing.clone()));
    }
    let receipt = DidChainSubmissionReceipt {
        provider: provider.to_owned(),
        transaction_id,
    };
    receipts_by_key.insert(idempotency_key.to_owned(), receipt.clone());
    Ok(DidChainSubmissionOutcome::Submitted(receipt))
}
