//! Fallback runtime-commit transport implementation used when `live-https` is disabled.

use super::{
    is_kolme_valid_http_transport_timeout_seconds_contract, parse_kolme_authorization_header_value,
    KamnKolmeTransportRequestPolicyError, KolmeApiBroadcastRequest, KolmeApiBroadcastResponse,
    KolmeApiNextNonceRequest, KolmeApiNextNonceResponse, KolmeRuntimeCommitError,
};
use kamn_kolme::{
    KolmeRuntimeCommitBlockFallbackTransport, KolmeRuntimeCommitFinalityTransport,
    KolmeRuntimeCommitProviderError, KolmeRuntimeCommitProviderTransport,
};

const LIVE_HTTPS_DISABLED_REASON: &str =
    "live-https transport feature disabled; rebuild kamn-core with --features live-https";

/// Runtime-commit HTTP transport facade for local-only builds with `live-https` disabled.
///
/// This type is intentionally fail-closed for network operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeRuntimeCommitHttpTransport {
    timeout_seconds: u64,
    authorization_header: Option<String>,
}

impl KolmeRuntimeCommitHttpTransport {
    /// Builds the disabled transport facade with deterministic timeout validation.
    pub fn new(timeout_seconds: u64) -> Result<Self, KolmeRuntimeCommitError> {
        if !is_kolme_valid_http_transport_timeout_seconds_contract(timeout_seconds) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "transport_timeout_seconds",
                reason: "must be positive",
            });
        }
        Ok(Self {
            timeout_seconds,
            authorization_header: None,
        })
    }

    /// Builds the disabled transport facade with authorization header validation.
    pub fn new_with_authorization(
        timeout_seconds: u64,
        authorization_header: &str,
    ) -> Result<Self, KolmeRuntimeCommitError> {
        let mut transport = Self::new(timeout_seconds)?;
        transport.authorization_header = Some(
            parse_kolme_authorization_header_value(authorization_header).map_err(|error| {
                match error {
                    KamnKolmeTransportRequestPolicyError::InvalidRequest { field, reason } => {
                        KolmeRuntimeCommitError::InvalidRequest { field, reason }
                    }
                }
            })?,
        );
        Ok(transport)
    }

    /// Returns an unavailable error because live HTTPS transport is disabled.
    pub fn fetch_next_nonce(
        &mut self,
        _base_url: &str,
        _nonce_path: &str,
        _request: &KolmeApiNextNonceRequest,
    ) -> Result<KolmeApiNextNonceResponse, KolmeRuntimeCommitProviderError> {
        Err(Self::live_https_disabled_error())
    }

    /// Returns an unavailable error because live HTTPS transport is disabled.
    pub fn submit_broadcast_request(
        &mut self,
        _base_url: &str,
        _submit_path: &str,
        _request: &KolmeApiBroadcastRequest,
        _idempotency_key: &str,
    ) -> Result<KolmeApiBroadcastResponse, KolmeRuntimeCommitProviderError> {
        Err(Self::live_https_disabled_error())
    }

    fn live_https_disabled_error() -> KolmeRuntimeCommitProviderError {
        KolmeRuntimeCommitProviderError::Unavailable {
            reason: LIVE_HTTPS_DISABLED_REASON.to_owned(),
        }
    }
}

impl KolmeRuntimeCommitProviderTransport for KolmeRuntimeCommitHttpTransport {
    fn submit_runtime_commit(
        &mut self,
        _base_url: &str,
        _submit_path: &str,
        _wire_payload: &str,
        _idempotency_key: &str,
    ) -> Result<String, KolmeRuntimeCommitProviderError> {
        Err(Self::live_https_disabled_error())
    }
}

impl KolmeRuntimeCommitFinalityTransport for KolmeRuntimeCommitHttpTransport {
    fn fetch_runtime_commit_finality(
        &mut self,
        _base_url: &str,
        _status_path: &str,
        _commit_id: &str,
    ) -> Result<String, KolmeRuntimeCommitProviderError> {
        Err(Self::live_https_disabled_error())
    }
}

impl KolmeRuntimeCommitBlockFallbackTransport for KolmeRuntimeCommitHttpTransport {
    fn fetch_block_by_height(
        &mut self,
        _base_url: &str,
        _block_path_template: &str,
        _height: u64,
    ) -> Result<String, KolmeRuntimeCommitProviderError> {
        Err(Self::live_https_disabled_error())
    }
}
