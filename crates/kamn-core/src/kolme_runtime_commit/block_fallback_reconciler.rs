//! Deterministic `/block/{height}` fallback reconciler for missed notification windows.

use super::{
    commit_finality_from_receipt_finality_contract,
    compose_kolme_block_fallback_unresolved_reason_contract,
    is_kolme_valid_block_fallback_base_url_input_contract,
    is_kolme_valid_block_fallback_lookup_budget_contract,
    is_kolme_valid_block_fallback_provider_input_contract,
    normalize_kolme_block_fallback_constructor_inputs_contract,
    parse_kolme_provider_block_fallback_response_contract,
    project_kolme_failed_block_txhash_receipt_contract,
    project_kolme_finalized_block_txhash_receipt_contract, validate_kolme_block_identity,
    validate_kolme_block_path_template, validate_kolme_lookup_txhash_contract,
    validate_kolme_lookup_window, BlockScanPolicyError, KolmeRuntimeCommitBlockFallbackTransport,
    KolmeRuntimeCommitError, KolmeRuntimeCommitProviderError, KolmeRuntimeCommitProviderReceipt,
    KolmeRuntimeCommitTransportErrorKind,
};

/// Deterministic `/block/{height}` fallback reconciler for missed notification windows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeRuntimeCommitBlockFallbackReconciler<T> {
    base_url: String,
    block_path_template: String,
    provider: String,
    max_block_lookups: u64,
    transport: T,
}

impl<T: KolmeRuntimeCommitBlockFallbackTransport> KolmeRuntimeCommitBlockFallbackReconciler<T> {
    /// Builds a block-fallback reconciler with deterministic validation.
    pub fn new(
        base_url: &str,
        block_path_template: &str,
        provider: &str,
        max_block_lookups: u64,
        transport: T,
    ) -> Result<Self, KolmeRuntimeCommitError> {
        if !is_kolme_valid_block_fallback_base_url_input_contract(base_url) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider_base_url",
                reason: "must not be empty",
            });
        }
        if !is_kolme_valid_block_fallback_provider_input_contract(provider) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider",
                reason: "must not be empty",
            });
        }
        if !is_kolme_valid_block_fallback_lookup_budget_contract(max_block_lookups) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "max_block_lookups",
                reason: "must be positive",
            });
        }
        validate_kolme_block_path_template(block_path_template)
            .map_err(|error| KolmeRuntimeCommitProviderError::Unavailable {
                reason: error.to_string(),
            })
            .map_err(|error| match error {
                KolmeRuntimeCommitProviderError::Timeout => {
                    KolmeRuntimeCommitError::ProviderTransport {
                        kind: KolmeRuntimeCommitTransportErrorKind::Timeout,
                        detail: "provider request timed out".to_owned(),
                    }
                }
                KolmeRuntimeCommitProviderError::Unavailable { reason } => {
                    KolmeRuntimeCommitError::ProviderTransport {
                        kind: KolmeRuntimeCommitTransportErrorKind::Unavailable,
                        detail: reason,
                    }
                }
                KolmeRuntimeCommitProviderError::MalformedResponse { reason } => {
                    KolmeRuntimeCommitError::ProviderTransport {
                        kind: KolmeRuntimeCommitTransportErrorKind::MalformedResponse,
                        detail: reason,
                    }
                }
            })?;
        let (base_url, block_path_template, provider) =
            normalize_kolme_block_fallback_constructor_inputs_contract(
                base_url,
                block_path_template,
                provider,
            );
        Ok(Self {
            base_url,
            block_path_template,
            provider,
            max_block_lookups,
            transport,
        })
    }

    /// Reconciles one tx hash by scanning block responses in the provided height window.
    pub fn reconcile_txhash(
        &mut self,
        txhash: &str,
        from_height: u64,
        latest_height: u64,
    ) -> Result<KolmeRuntimeCommitProviderReceipt, KolmeRuntimeCommitProviderError> {
        let txhash = validate_kolme_lookup_txhash_contract(txhash).map_err(|error| {
            KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: error.to_string(),
            }
        })?;
        validate_kolme_lookup_window(from_height, latest_height, self.max_block_lookups).map_err(
            |error| match error {
                BlockScanPolicyError::MaxLookupsExceeded { .. } => {
                    KolmeRuntimeCommitProviderError::Unavailable {
                        reason: error.to_string(),
                    }
                }
                _ => KolmeRuntimeCommitProviderError::MalformedResponse {
                    reason: error.to_string(),
                },
            },
        )?;

        for height in from_height..=latest_height {
            let response = self.transport.fetch_block_by_height(
                self.base_url.as_str(),
                self.block_path_template.as_str(),
                height,
            )?;
            let block = parse_kolme_provider_block_fallback_response_contract(
                response.as_str(),
                self.provider.as_str(),
                height,
            )
            .map_err(|error| KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: error.to_string(),
            })?;

            validate_kolme_block_identity(
                self.provider.as_str(),
                block.provider.as_str(),
                height,
                block.block_height,
            )
            .map_err(|error| KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: error.to_string(),
            })?;

            if block
                .finalized_tx_hashes
                .iter()
                .any(|value| value == txhash.as_str())
            {
                let projection =
                    project_kolme_finalized_block_txhash_receipt_contract(txhash.as_str(), height);
                return Ok(KolmeRuntimeCommitProviderReceipt {
                    provider: self.provider.clone(),
                    commit_id: projection.commit_id,
                    finality: commit_finality_from_receipt_finality_contract(projection.finality),
                });
            }
            if block
                .failed_tx_hashes
                .iter()
                .any(|value| value == txhash.as_str())
            {
                let projection =
                    project_kolme_failed_block_txhash_receipt_contract(txhash.as_str());
                return Ok(KolmeRuntimeCommitProviderReceipt {
                    provider: self.provider.clone(),
                    commit_id: projection.commit_id,
                    finality: commit_finality_from_receipt_finality_contract(projection.finality),
                });
            }
        }

        Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: compose_kolme_block_fallback_unresolved_reason_contract(
                txhash.as_str(),
                from_height,
                latest_height,
            ),
        })
    }
}
