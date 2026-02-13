//! Deterministic finality checker for live backend runtime commit receipts.

use super::{
    is_kolme_terminal_receipt_finality_contract, is_kolme_valid_finality_base_url_input_contract,
    is_kolme_valid_finality_status_path_input_contract,
    is_kolme_valid_poll_attempt_budget_contract, is_kolme_valid_runtime_commit_id_request_contract,
    normalize_kolme_finality_endpoint_inputs_contract, parse_kolme_provider_finality_receipt,
    KolmeRuntimeCommitError, KolmeRuntimeCommitProviderReceipt,
};
use kamn_kolme::{KolmeRuntimeCommitFinalityTransport, KolmeRuntimeCommitProviderError};

/// Deterministic finality checker for live backend runtime commit receipts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeRuntimeCommitFinalityChecker<T> {
    base_url: String,
    status_path: String,
    transport: T,
}

impl<T: KolmeRuntimeCommitFinalityTransport> KolmeRuntimeCommitFinalityChecker<T> {
    /// Builds a finality checker with deterministic endpoint validation.
    pub fn new(
        base_url: &str,
        status_path: &str,
        transport: T,
    ) -> Result<Self, KolmeRuntimeCommitError> {
        if !is_kolme_valid_finality_base_url_input_contract(base_url) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider_base_url",
                reason: "must not be empty",
            });
        }
        if !is_kolme_valid_finality_status_path_input_contract(status_path) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider_status_path",
                reason: "must not be empty",
            });
        }
        let (base_url, status_path) =
            normalize_kolme_finality_endpoint_inputs_contract(base_url, status_path);
        Ok(Self {
            base_url,
            status_path,
            transport,
        })
    }

    /// Fetches and parses one backend finality response for the provided commit.
    pub fn check_commit_finality(
        &mut self,
        commit_id: &str,
    ) -> Result<KolmeRuntimeCommitProviderReceipt, KolmeRuntimeCommitProviderError> {
        if !is_kolme_valid_runtime_commit_id_request_contract(commit_id) {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "commit_id must not be empty".to_owned(),
            });
        }

        let response = self.transport.fetch_runtime_commit_finality(
            self.base_url.as_str(),
            self.status_path.as_str(),
            commit_id,
        )?;
        let receipt = parse_kolme_provider_finality_receipt(response.as_str(), commit_id).map_err(
            |error| KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: error.to_string(),
            },
        )?;
        Ok(KolmeRuntimeCommitProviderReceipt {
            provider: receipt.provider,
            commit_id: receipt.commit_id,
            finality: receipt.finality,
        })
    }

    /// Polls backend finality and returns the first non-pending receipt.
    pub fn poll_finality(
        &mut self,
        commit_id: &str,
        max_attempts: u32,
    ) -> Result<KolmeRuntimeCommitProviderReceipt, KolmeRuntimeCommitProviderError> {
        if !is_kolme_valid_poll_attempt_budget_contract(max_attempts) {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "max_attempts must be positive".to_owned(),
            });
        }
        for _ in 0..max_attempts {
            let receipt = self.check_commit_finality(commit_id)?;
            if is_kolme_terminal_receipt_finality_contract(receipt.finality) {
                return Ok(receipt);
            }
        }
        Err(KolmeRuntimeCommitProviderError::Timeout)
    }
}
