//! Runtime-commit interface and transport trait ownership.

use super::{
    KolmeRuntimeCommitError, KolmeRuntimeCommitOutcome, KolmeRuntimeCommitProviderError,
    KolmeRuntimeCommitProviderOutcome, KolmeRuntimeCommitRequest,
};

/// Abstract client interface for Kolme runtime commit submission.
pub trait KolmeRuntimeCommitClient {
    /// Submits one deterministic runtime commit request.
    fn submit_commit(
        &mut self,
        request: &KolmeRuntimeCommitRequest,
    ) -> Result<KolmeRuntimeCommitOutcome, KolmeRuntimeCommitError>;
}

/// Provider interface consumed by the adapter-backed runtime commit client.
pub trait KolmeRuntimeCommitProvider {
    /// Submits canonical wire payload with deterministic idempotency key.
    fn submit_runtime_commit(
        &mut self,
        wire_payload: &str,
        idempotency_key: &str,
    ) -> Result<KolmeRuntimeCommitProviderOutcome, KolmeRuntimeCommitProviderError>;
}
