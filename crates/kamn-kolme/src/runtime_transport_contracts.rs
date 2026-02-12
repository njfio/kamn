//! Runtime transport-facing contracts shared across Kolme runtime adapters.

use crate::KolmeTransportIoClassification;
use std::fmt;

/// Typed transport error class emitted when provider transport calls fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KolmeRuntimeCommitTransportErrorKind {
    /// Provider call timed out.
    Timeout,
    /// Provider transport/channel is unavailable.
    Unavailable,
    /// Provider response payload is malformed.
    MalformedResponse,
}

/// Provider-facing error for runtime commit adapter wiring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeRuntimeCommitProviderError {
    /// Provider call timed out before a response.
    Timeout,
    /// Provider transport/channel is unavailable.
    Unavailable {
        /// Provider-specific availability failure reason.
        reason: String,
    },
    /// Provider emitted malformed payload/shape.
    MalformedResponse {
        /// Provider-specific malformed payload reason.
        reason: String,
    },
}

impl fmt::Display for KolmeRuntimeCommitProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout => write!(f, "provider request timed out"),
            Self::Unavailable { reason } => write!(f, "provider unavailable: {reason}"),
            Self::MalformedResponse { reason } => {
                write!(f, "provider malformed response: {reason}")
            }
        }
    }
}

impl std::error::Error for KolmeRuntimeCommitProviderError {}

impl From<KolmeTransportIoClassification> for KolmeRuntimeCommitProviderError {
    fn from(value: KolmeTransportIoClassification) -> Self {
        match value {
            KolmeTransportIoClassification::Timeout => Self::Timeout,
            KolmeTransportIoClassification::Unavailable { reason } => Self::Unavailable { reason },
        }
    }
}

/// Transport connection abstraction for consuming notifications text messages.
pub trait KolmeRuntimeCommitNotificationsConnection {
    /// Reads the next notifications text message.
    ///
    /// Returns `Ok(None)` when the current websocket connection is closed.
    fn read_text_message(&mut self) -> Result<Option<String>, KolmeRuntimeCommitProviderError>;
}

/// Connector abstraction for establishing notifications websocket connections.
pub trait KolmeRuntimeCommitNotificationsConnector {
    /// Concrete connection type returned by the connector.
    type Connection: KolmeRuntimeCommitNotificationsConnection;

    /// Connects to one websocket notifications URL.
    fn connect(
        &mut self,
        notifications_url: &str,
    ) -> Result<Self::Connection, KolmeRuntimeCommitProviderError>;
}

/// Transport abstraction used by the live provider bridge to reach Kolme backends.
pub trait KolmeRuntimeCommitProviderTransport {
    /// Submits one runtime commit payload to the configured provider endpoint.
    fn submit_runtime_commit(
        &mut self,
        base_url: &str,
        submit_path: &str,
        wire_payload: &str,
        idempotency_key: &str,
    ) -> Result<String, KolmeRuntimeCommitProviderError>;
}

/// Transport abstraction for querying runtime commit finality from a live backend.
pub trait KolmeRuntimeCommitFinalityTransport {
    /// Fetches one finality response payload for the provided commit identifier.
    fn fetch_runtime_commit_finality(
        &mut self,
        base_url: &str,
        status_path: &str,
        commit_id: &str,
    ) -> Result<String, KolmeRuntimeCommitProviderError>;
}

/// Transport abstraction for querying `/block/{height}` fallback responses.
pub trait KolmeRuntimeCommitBlockFallbackTransport {
    /// Fetches one block response payload for the provided height.
    fn fetch_block_by_height(
        &mut self,
        base_url: &str,
        block_path_template: &str,
        height: u64,
    ) -> Result<String, KolmeRuntimeCommitProviderError>;
}
