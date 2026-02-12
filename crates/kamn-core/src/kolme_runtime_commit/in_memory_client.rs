//! Deterministic in-memory runtime commit client for tests and local development.

use super::{
    deterministic_runtime_commit_id_contract, is_kolme_valid_runtime_provider_input_contract,
    KolmeCommitReceiptFinality, KolmeRuntimeCommitClient, KolmeRuntimeCommitError,
    KolmeRuntimeCommitOutcome, KolmeRuntimeCommitReceipt, KolmeRuntimeCommitRequest,
};
use std::collections::HashMap;

/// Deterministic in-memory commit client used for contract tests and local development.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InMemoryKolmeRuntimeCommitClient {
    provider: String,
    receipts_by_idempotency_key: HashMap<String, KolmeRuntimeCommitReceipt>,
    finality_by_idempotency_key: HashMap<String, KolmeCommitReceiptFinality>,
    rejected_reasons_by_idempotency_key: HashMap<String, String>,
}

impl InMemoryKolmeRuntimeCommitClient {
    /// Constructs an in-memory commit client.
    pub fn new(provider: &str) -> Result<Self, KolmeRuntimeCommitError> {
        if !is_kolme_valid_runtime_provider_input_contract(provider) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider",
                reason: "must not be empty",
            });
        }
        Ok(Self {
            provider: provider.to_owned(),
            receipts_by_idempotency_key: HashMap::new(),
            finality_by_idempotency_key: HashMap::new(),
            rejected_reasons_by_idempotency_key: HashMap::new(),
        })
    }

    /// Forces deterministic rejection for the provided idempotency key.
    pub fn reject_idempotency_key(&mut self, idempotency_key: &str, reason: &str) {
        self.rejected_reasons_by_idempotency_key
            .insert(idempotency_key.to_owned(), reason.to_owned());
    }

    /// Overrides the receipt finality that will be emitted for a given idempotency key.
    pub fn set_finality_for_idempotency_key(
        &mut self,
        idempotency_key: &str,
        finality: KolmeCommitReceiptFinality,
    ) {
        self.finality_by_idempotency_key
            .insert(idempotency_key.to_owned(), finality);
    }
}

impl KolmeRuntimeCommitClient for InMemoryKolmeRuntimeCommitClient {
    fn submit_commit(
        &mut self,
        request: &KolmeRuntimeCommitRequest,
    ) -> Result<KolmeRuntimeCommitOutcome, KolmeRuntimeCommitError> {
        request.validate()?;

        if let Some(reason) = self
            .rejected_reasons_by_idempotency_key
            .get(request.idempotency_key())
        {
            return Ok(KolmeRuntimeCommitOutcome::Rejected {
                reason: reason.clone(),
            });
        }

        if let Some(existing) = self
            .receipts_by_idempotency_key
            .get(request.idempotency_key())
        {
            return Ok(KolmeRuntimeCommitOutcome::Duplicate(existing.clone()));
        }

        let receipt = KolmeRuntimeCommitReceipt {
            provider: self.provider.clone(),
            commit_id: deterministic_runtime_commit_id_contract(
                request.operation_id.as_str(),
                request.actor_did.as_str(),
                request.nonce,
                request.payload_hash.as_str(),
            ),
            finality: self
                .finality_by_idempotency_key
                .get(request.idempotency_key())
                .copied()
                .unwrap_or(KolmeCommitReceiptFinality::Pending),
        };

        self.receipts_by_idempotency_key
            .insert(request.idempotency_key().to_owned(), receipt.clone());
        Ok(KolmeRuntimeCommitOutcome::Submitted(receipt))
    }
}
