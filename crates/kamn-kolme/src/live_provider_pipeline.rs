//! Live-provider pipeline contracts for runtime-commit submission.

use crate::{
    is_valid_live_provider_base_url_input, is_valid_live_provider_submit_path_input,
    is_valid_provider_hint_input, normalize_live_provider_endpoint_inputs,
    normalize_provider_hint_input, parse_live_runtime_provider_outcome,
    KolmeRuntimeCommitProviderError, KolmeRuntimeCommitProviderTransport,
    KolmeRuntimeProviderOutcome,
};
use std::fmt;

/// Submission profile used by the live runtime-commit provider pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KolmeRuntimeCommitLiveProviderProfile {
    /// Legacy submit/finality endpoints.
    LegacyRuntimeCommit,
    /// `kolme_fork` broadcast endpoint profile.
    KolmeForkBroadcast,
}

/// Normalized live-provider configuration used for runtime-commit submission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeRuntimeCommitLiveProviderConfig {
    base_url: String,
    submit_path: String,
    profile: KolmeRuntimeCommitLiveProviderProfile,
    provider_hint: Option<String>,
}

impl KolmeRuntimeCommitLiveProviderConfig {
    /// Returns normalized base URL.
    pub fn base_url(&self) -> &str {
        self.base_url.as_str()
    }

    /// Returns normalized submit path.
    pub fn submit_path(&self) -> &str {
        self.submit_path.as_str()
    }

    /// Returns configured submission profile.
    pub fn profile(&self) -> KolmeRuntimeCommitLiveProviderProfile {
        self.profile
    }

    /// Returns optional provider hint used for fork broadcast profile parsing.
    pub fn provider_hint(&self) -> Option<&str> {
        self.provider_hint.as_deref()
    }
}

/// Deterministic config error emitted while building live-provider configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeRuntimeCommitLiveProviderConfigError {
    /// Invalid config input field.
    InvalidRequest {
        /// Field failing validation.
        field: &'static str,
        /// Validation reason.
        reason: &'static str,
    },
}

impl fmt::Display for KolmeRuntimeCommitLiveProviderConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest { field, reason } => {
                write!(f, "invalid live provider config {field}: {reason}")
            }
        }
    }
}

impl std::error::Error for KolmeRuntimeCommitLiveProviderConfigError {}

/// Builds normalized live-provider configuration for legacy runtime-commit submit semantics.
pub fn build_runtime_commit_live_provider_config(
    base_url: &str,
    submit_path: &str,
) -> Result<KolmeRuntimeCommitLiveProviderConfig, KolmeRuntimeCommitLiveProviderConfigError> {
    if !is_valid_live_provider_base_url_input(base_url) {
        return Err(KolmeRuntimeCommitLiveProviderConfigError::InvalidRequest {
            field: "provider_base_url",
            reason: "must not be empty",
        });
    }
    if !is_valid_live_provider_submit_path_input(submit_path) {
        return Err(KolmeRuntimeCommitLiveProviderConfigError::InvalidRequest {
            field: "provider_submit_path",
            reason: "must not be empty",
        });
    }
    let (base_url, submit_path) = normalize_live_provider_endpoint_inputs(base_url, submit_path);
    Ok(KolmeRuntimeCommitLiveProviderConfig {
        base_url,
        submit_path,
        profile: KolmeRuntimeCommitLiveProviderProfile::LegacyRuntimeCommit,
        provider_hint: None,
    })
}

/// Builds normalized `kolme_fork` broadcast configuration for runtime-commit submit semantics.
pub fn build_kolme_fork_broadcast_live_provider_config(
    base_url: &str,
    provider_hint: &str,
) -> Result<KolmeRuntimeCommitLiveProviderConfig, KolmeRuntimeCommitLiveProviderConfigError> {
    if !is_valid_provider_hint_input(provider_hint) {
        return Err(KolmeRuntimeCommitLiveProviderConfigError::InvalidRequest {
            field: "provider_hint",
            reason: "must not be empty",
        });
    }
    let provider_hint = normalize_provider_hint_input(provider_hint);
    let mut config = build_runtime_commit_live_provider_config(base_url, "/broadcast")?;
    config.profile = KolmeRuntimeCommitLiveProviderProfile::KolmeForkBroadcast;
    config.provider_hint = Some(provider_hint.to_owned());
    Ok(config)
}

/// Submits one runtime commit payload through provider transport using normalized live config.
pub fn submit_runtime_commit_live_provider_request<T: KolmeRuntimeCommitProviderTransport>(
    transport: &mut T,
    config: &KolmeRuntimeCommitLiveProviderConfig,
    wire_payload: &str,
    idempotency_key: &str,
) -> Result<KolmeRuntimeProviderOutcome, KolmeRuntimeCommitProviderError> {
    let response = transport.submit_runtime_commit(
        config.base_url.as_str(),
        config.submit_path.as_str(),
        wire_payload,
        idempotency_key,
    )?;
    let provider_hint = match config.profile {
        KolmeRuntimeCommitLiveProviderProfile::KolmeForkBroadcast => {
            config.provider_hint.as_deref()
        }
        KolmeRuntimeCommitLiveProviderProfile::LegacyRuntimeCommit => None,
    };
    parse_live_runtime_provider_outcome(response.as_str(), provider_hint).map_err(|error| {
        KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: error.to_string(),
        }
    })
}
