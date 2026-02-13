//! Provider implementation that bridges runtime commit requests through a live transport.

use super::{
    build_kamn_kolme_fork_broadcast_live_provider_config,
    build_kamn_kolme_runtime_commit_live_provider_config,
    submit_kamn_kolme_runtime_commit_live_provider_request,
    KamnKolmeRuntimeCommitLiveProviderConfig, KamnKolmeRuntimeCommitLiveProviderConfigError,
    KolmeRuntimeCommitError, KolmeRuntimeCommitProvider, KolmeRuntimeCommitProviderOutcome,
};
use kamn_kolme::{KolmeRuntimeCommitProviderError, KolmeRuntimeCommitProviderTransport};

/// Provider implementation that bridges runtime commit requests through a live transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeRuntimeCommitLiveProvider<T> {
    config: KamnKolmeRuntimeCommitLiveProviderConfig,
    transport: T,
}

fn map_live_provider_config_error(
    error: KamnKolmeRuntimeCommitLiveProviderConfigError,
) -> KolmeRuntimeCommitError {
    match error {
        KamnKolmeRuntimeCommitLiveProviderConfigError::InvalidRequest { field, reason } => {
            KolmeRuntimeCommitError::InvalidRequest { field, reason }
        }
    }
}

impl<T: KolmeRuntimeCommitProviderTransport> KolmeRuntimeCommitLiveProvider<T> {
    /// Builds a live provider with deterministic endpoint validation.
    pub fn new(
        base_url: &str,
        submit_path: &str,
        transport: T,
    ) -> Result<Self, KolmeRuntimeCommitError> {
        let config = build_kamn_kolme_runtime_commit_live_provider_config(base_url, submit_path)
            .map_err(map_live_provider_config_error)?;
        Ok(Self { config, transport })
    }

    /// Builds a live provider configured for `kolme_fork` broadcast semantics.
    pub fn new_kolme_fork_broadcast_profile(
        base_url: &str,
        provider_hint: &str,
        transport: T,
    ) -> Result<Self, KolmeRuntimeCommitError> {
        let config = build_kamn_kolme_fork_broadcast_live_provider_config(base_url, provider_hint)
            .map_err(map_live_provider_config_error)?;
        Ok(Self { config, transport })
    }
}

impl<T: KolmeRuntimeCommitProviderTransport> KolmeRuntimeCommitProvider
    for KolmeRuntimeCommitLiveProvider<T>
{
    fn submit_runtime_commit(
        &mut self,
        wire_payload: &str,
        idempotency_key: &str,
    ) -> Result<KolmeRuntimeCommitProviderOutcome, KolmeRuntimeCommitProviderError> {
        let outcome = submit_kamn_kolme_runtime_commit_live_provider_request(
            &mut self.transport,
            &self.config,
            wire_payload,
            idempotency_key,
        )?;
        Ok(outcome.into())
    }
}
