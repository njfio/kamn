//! Adapter-backed runtime commit client ownership.

use super::{
    is_kolme_valid_expected_provider_input_contract, require_kolme_final_receipt_finality_contract,
    validate_kolme_provider_receipt_identity_contract, KamnKolmeProviderReceiptIdentityError,
    KamnKolmeRuntimeLifecyclePolicyError, KolmeRuntimeCommitClient, KolmeRuntimeCommitError,
    KolmeRuntimeCommitOutcome, KolmeRuntimeCommitProvider, KolmeRuntimeCommitProviderOutcome,
    KolmeRuntimeCommitProviderReceipt, KolmeRuntimeCommitReceipt, KolmeRuntimeCommitRequest,
};
use kamn_kolme::{KolmeRuntimeCommitProviderError, KolmeRuntimeCommitTransportErrorKind};

/// Adapter-backed runtime commit client that enforces provider and finality policies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterBackedKolmeRuntimeCommitClient<P> {
    expected_provider: String,
    provider: P,
}

impl<P: KolmeRuntimeCommitProvider> AdapterBackedKolmeRuntimeCommitClient<P> {
    /// Builds adapter-backed client with expected provider identifier.
    pub fn new(expected_provider: &str, provider: P) -> Result<Self, KolmeRuntimeCommitError> {
        if !is_kolme_valid_expected_provider_input_contract(expected_provider) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "expected_provider",
                reason: "must not be empty",
            });
        }
        Ok(Self {
            expected_provider: expected_provider.to_owned(),
            provider,
        })
    }
}

impl<P: KolmeRuntimeCommitProvider> KolmeRuntimeCommitClient
    for AdapterBackedKolmeRuntimeCommitClient<P>
{
    fn submit_commit(
        &mut self,
        request: &KolmeRuntimeCommitRequest,
    ) -> Result<KolmeRuntimeCommitOutcome, KolmeRuntimeCommitError> {
        request.validate()?;
        let expected_provider = self.expected_provider.as_str();
        let map_provider_receipt = |receipt: KolmeRuntimeCommitProviderReceipt| {
            validate_kolme_provider_receipt_identity_contract(
                expected_provider,
                receipt.provider.as_str(),
                receipt.commit_id.as_str(),
            )
            .map_err(|error| match error {
                KamnKolmeProviderReceiptIdentityError::ProviderMismatch { expected, observed } => {
                    KolmeRuntimeCommitError::ProviderMismatch { expected, observed }
                }
                KamnKolmeProviderReceiptIdentityError::EmptyCommitId => {
                    KolmeRuntimeCommitError::InvalidRequest {
                        field: "receipt_commit_id",
                        reason: "must not be empty",
                    }
                }
            })?;
            require_kolme_final_receipt_finality_contract(receipt.finality).map_err(|error| {
                match error {
                    KamnKolmeRuntimeLifecyclePolicyError::NonFinalReceipt { finality } => {
                        KolmeRuntimeCommitError::NonFinalReceipt {
                            commit_id: receipt.commit_id.clone(),
                            finality,
                        }
                    }
                }
            })?;
            Ok(KolmeRuntimeCommitReceipt {
                provider: receipt.provider,
                commit_id: receipt.commit_id,
                finality: receipt.finality,
            })
        };
        let provider_outcome = self
            .provider
            .submit_runtime_commit(&request.to_wire_payload(), request.idempotency_key())
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
        match provider_outcome {
            KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => Ok(
                KolmeRuntimeCommitOutcome::Submitted(map_provider_receipt(receipt)?),
            ),
            KolmeRuntimeCommitProviderOutcome::Duplicate(receipt) => Ok(
                KolmeRuntimeCommitOutcome::Duplicate(map_provider_receipt(receipt)?),
            ),
            KolmeRuntimeCommitProviderOutcome::Rejected { reason } => {
                Ok(KolmeRuntimeCommitOutcome::Rejected { reason })
            }
        }
    }
}
