use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Kolme-backed DID lifecycle chain adapter.
pub struct KolmeDidLifecycleChainAdapter<C> {
    client: C,
    state_root_prefix: String,
}

impl<C> KolmeDidLifecycleChainAdapter<C> {
    /// Creates a Kolme-backed lifecycle adapter.
    pub fn new(client: C, state_root_prefix: &str) -> Result<Self, DidRegistryError> {
        if state_root_prefix.trim().is_empty() {
            return Err(DidRegistryError::ChainAdapterSubmitFailed {
                context: "state_root_prefix",
                reason: "must not be empty".to_owned(),
            });
        }
        Ok(Self {
            client,
            state_root_prefix: state_root_prefix.to_owned(),
        })
    }

    fn runtime_request_for_lifecycle(
        &self,
        request: &DidLifecycleChainSubmissionRequest,
    ) -> Result<KolmeRuntimeCommitRequest, DidRegistryError> {
        let operation_id = format!(
            "did-lifecycle:{}:{}:{}",
            request.did.method_specific_id(),
            request.action,
            request.nonce
        );
        let state_root = format!("{}:{}", self.state_root_prefix, request.nonce);
        KolmeRuntimeCommitRequest::deterministic(
            operation_id.as_str(),
            state_root.as_str(),
            request.did.as_str(),
            request.nonce,
            request.payload_hash.as_str(),
        )
        .map_err(Self::map_runtime_commit_error)
    }

    fn map_runtime_commit_error(error: KolmeRuntimeCommitError) -> DidRegistryError {
        match error {
            KolmeRuntimeCommitError::InvalidRequest { field, reason } => {
                DidRegistryError::ChainAdapterSubmitFailed {
                    context: field,
                    reason: reason.to_owned(),
                }
            }
            _ => DidRegistryError::ChainAdapterSubmitFailed {
                context: "kolme_runtime_commit",
                reason: error.to_string(),
            },
        }
    }
}

impl<C: KolmeRuntimeCommitClient> DidLifecycleChainAdapter for KolmeDidLifecycleChainAdapter<C> {
    fn submit_lifecycle_mutation(
        &mut self,
        request: &DidLifecycleChainSubmissionRequest,
    ) -> Result<DidChainSubmissionOutcome, DidRegistryError> {
        let runtime_request = self.runtime_request_for_lifecycle(request)?;
        let outcome = self
            .client
            .submit_commit(&runtime_request)
            .map_err(Self::map_runtime_commit_error)?;
        match outcome {
            KolmeRuntimeCommitOutcome::Submitted(receipt) => Ok(
                DidChainSubmissionOutcome::Submitted(DidChainSubmissionReceipt {
                    provider: receipt.provider,
                    transaction_id: receipt.commit_id,
                }),
            ),
            KolmeRuntimeCommitOutcome::Duplicate(receipt) => Ok(
                DidChainSubmissionOutcome::Duplicate(DidChainSubmissionReceipt {
                    provider: receipt.provider,
                    transaction_id: receipt.commit_id,
                }),
            ),
            KolmeRuntimeCommitOutcome::Rejected { reason } => {
                Ok(DidChainSubmissionOutcome::Rejected { reason })
            }
        }
    }
}
