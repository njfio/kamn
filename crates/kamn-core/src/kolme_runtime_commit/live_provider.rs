//! Provider implementation that bridges runtime commit requests through a live transport.

use super::{
    is_kolme_valid_live_provider_base_url_input_contract,
    is_kolme_valid_live_provider_submit_path_input_contract,
    is_kolme_valid_provider_hint_input_contract,
    normalize_kolme_live_provider_endpoint_inputs_contract,
    normalize_kolme_provider_hint_input_contract,
    parse_kolme_live_runtime_provider_outcome_contract, KolmeRuntimeCommitError,
    KolmeRuntimeCommitProvider, KolmeRuntimeCommitProviderError, KolmeRuntimeCommitProviderOutcome,
    KolmeRuntimeCommitProviderTransport,
};

/// Provider implementation that bridges runtime commit requests through a live transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeRuntimeCommitLiveProvider<T> {
    base_url: String,
    submit_path: String,
    profile: KolmeRuntimeCommitSubmitProfile,
    provider_hint: Option<String>,
    transport: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KolmeRuntimeCommitSubmitProfile {
    LegacyRuntimeCommit,
    KolmeForkBroadcast,
}

impl<T: KolmeRuntimeCommitProviderTransport> KolmeRuntimeCommitLiveProvider<T> {
    /// Builds a live provider with deterministic endpoint validation.
    pub fn new(
        base_url: &str,
        submit_path: &str,
        transport: T,
    ) -> Result<Self, KolmeRuntimeCommitError> {
        if !is_kolme_valid_live_provider_base_url_input_contract(base_url) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider_base_url",
                reason: "must not be empty",
            });
        }
        if !is_kolme_valid_live_provider_submit_path_input_contract(submit_path) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider_submit_path",
                reason: "must not be empty",
            });
        }
        let (base_url, submit_path) =
            normalize_kolme_live_provider_endpoint_inputs_contract(base_url, submit_path);
        Ok(Self {
            base_url,
            submit_path,
            profile: KolmeRuntimeCommitSubmitProfile::LegacyRuntimeCommit,
            provider_hint: None,
            transport,
        })
    }

    /// Builds a live provider configured for `kolme_fork` broadcast semantics.
    pub fn new_kolme_fork_broadcast_profile(
        base_url: &str,
        provider_hint: &str,
        transport: T,
    ) -> Result<Self, KolmeRuntimeCommitError> {
        if !is_kolme_valid_provider_hint_input_contract(provider_hint) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider_hint",
                reason: "must not be empty",
            });
        }
        let provider_hint = normalize_kolme_provider_hint_input_contract(provider_hint);
        let mut provider = Self::new(base_url, "/broadcast", transport)?;
        provider.profile = KolmeRuntimeCommitSubmitProfile::KolmeForkBroadcast;
        provider.provider_hint = Some(provider_hint.to_owned());
        Ok(provider)
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
        let response = self.transport.submit_runtime_commit(
            self.base_url.as_str(),
            self.submit_path.as_str(),
            wire_payload,
            idempotency_key,
        )?;
        let provider_hint = match self.profile {
            KolmeRuntimeCommitSubmitProfile::KolmeForkBroadcast => self.provider_hint.as_deref(),
            KolmeRuntimeCommitSubmitProfile::LegacyRuntimeCommit => None,
        };
        let outcome =
            parse_kolme_live_runtime_provider_outcome_contract(response.as_str(), provider_hint)
                .map_err(|error| KolmeRuntimeCommitProviderError::MalformedResponse {
                    reason: error.to_string(),
                })?;
        Ok(outcome.into())
    }
}
