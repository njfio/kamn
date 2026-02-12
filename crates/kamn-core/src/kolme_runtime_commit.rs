//! Deterministic runtime-commit request/receipt contracts for Kolme integration.

use crate::AgentDid;
use kamn_kolme::{
    are_runtime_commit_request_fields_single_line as are_kolme_runtime_commit_request_fields_single_line_contract,
    classify_tls_failure_reason as classify_kolme_tls_failure_reason,
    classify_transport_io_error as classify_kolme_transport_io_error,
    commit_finality_from_receipt_finality as commit_finality_from_receipt_finality_contract,
    commit_finality_label as commit_finality_label_contract,
    compose_block_fallback_unresolved_reason as compose_kolme_block_fallback_unresolved_reason_contract,
    compose_finality_status_path as compose_kolme_finality_status_path,
    compose_notifications_reconnect_exhausted_reason as compose_kolme_notifications_reconnect_exhausted_reason_contract,
    compose_notifications_websocket_url as compose_kolme_notifications_websocket_url,
    deterministic_runtime_commit_id as deterministic_runtime_commit_id_contract,
    deterministic_runtime_commit_idempotency_key as deterministic_runtime_commit_idempotency_key_contract,
    find_http_header_boundary as find_kolme_http_header_boundary,
    is_broadcast_submit_path as is_kolme_broadcast_submit_path_contract,
    is_canonical_runtime_commit_signed_message as is_kolme_canonical_runtime_commit_signed_message_contract,
    is_terminal_receipt_finality as is_kolme_terminal_receipt_finality_contract,
    is_valid_block_fallback_base_url_input as is_kolme_valid_block_fallback_base_url_input_contract,
    is_valid_block_fallback_lookup_budget as is_kolme_valid_block_fallback_lookup_budget_contract,
    is_valid_block_fallback_provider_input as is_kolme_valid_block_fallback_provider_input_contract,
    is_valid_block_lookup_height as is_kolme_valid_block_lookup_height_contract,
    is_valid_expected_provider_input as is_kolme_valid_expected_provider_input_contract,
    is_valid_finality_base_url_input as is_kolme_valid_finality_base_url_input_contract,
    is_valid_finality_status_path_input as is_kolme_valid_finality_status_path_input_contract,
    is_valid_http_response_bytes_input as is_kolme_valid_http_response_bytes_input_contract,
    is_valid_http_transport_timeout_seconds as is_kolme_valid_http_transport_timeout_seconds_contract,
    is_valid_live_provider_base_url_input as is_kolme_valid_live_provider_base_url_input_contract,
    is_valid_live_provider_submit_path_input as is_kolme_valid_live_provider_submit_path_input_contract,
    is_valid_notifications_provider_input as is_kolme_valid_notifications_provider_input_contract,
    is_valid_notifications_reconnect_budget as is_kolme_valid_notifications_reconnect_budget_contract,
    is_valid_poll_attempt_budget as is_kolme_valid_poll_attempt_budget_contract,
    is_valid_provider_hint_input as is_kolme_valid_provider_hint_input_contract,
    is_valid_receipt_commit_id_input as is_kolme_valid_receipt_commit_id_input_contract,
    is_valid_receipt_provider_input as is_kolme_valid_receipt_provider_input_contract,
    is_valid_runtime_commit_id_request as is_kolme_valid_runtime_commit_id_request_contract,
    is_valid_runtime_nonce_input as is_kolme_valid_runtime_nonce_input_contract,
    is_valid_runtime_operation_id_input as is_kolme_valid_runtime_operation_id_input_contract,
    is_valid_runtime_payload_hash_input as is_kolme_valid_runtime_payload_hash_input_contract,
    is_valid_runtime_provider_input as is_kolme_valid_runtime_provider_input_contract,
    is_valid_runtime_state_root_input as is_kolme_valid_runtime_state_root_input_contract,
    is_valid_signed_envelope_message_input as is_kolme_valid_signed_envelope_message_input_contract,
    is_valid_signed_envelope_signature_input as is_kolme_valid_signed_envelope_signature_input_contract,
    is_valid_signed_envelope_signer_key_id_input as is_kolme_valid_signed_envelope_signer_key_id_input_contract,
    is_valid_transport_idempotency_key_input as is_kolme_valid_transport_idempotency_key_input_contract,
    is_valid_transport_wire_payload_input as is_kolme_valid_transport_wire_payload_input_contract,
    is_valid_websocket_timeout_seconds as is_kolme_valid_websocket_timeout_seconds_contract,
    lifecycle_state_for_finality as lifecycle_state_for_finality_contract,
    lifecycle_state_label as lifecycle_state_label_contract,
    normalize_block_fallback_constructor_inputs as normalize_kolme_block_fallback_constructor_inputs_contract,
    normalize_broadcast_payload as normalize_kolme_broadcast_payload_contract,
    normalize_broadcast_submit_path_input as normalize_kolme_broadcast_submit_path_input_contract,
    normalize_finality_endpoint_inputs as normalize_kolme_finality_endpoint_inputs_contract,
    normalize_live_provider_endpoint_inputs as normalize_kolme_live_provider_endpoint_inputs_contract,
    normalize_notifications_provider_input as normalize_kolme_notifications_provider_input_contract,
    normalize_provider_hint_input as normalize_kolme_provider_hint_input_contract,
    normalize_runtime_commit_request_fields as normalize_kolme_runtime_commit_request_fields_contract,
    normalize_runtime_commit_signed_envelope_fields as normalize_kolme_runtime_commit_signed_envelope_fields_contract,
    normalize_transport_idempotency_key_input as normalize_kolme_transport_idempotency_key_input_contract,
    notification_event_to_provider_receipt as notification_event_to_kolme_provider_receipt_contract,
    parse_authorization_header_value as parse_kolme_authorization_header_value,
    parse_http_endpoint as parse_kolme_http_endpoint,
    parse_http_response_body as parse_kolme_http_response_body,
    parse_live_runtime_provider_outcome as parse_kolme_live_runtime_provider_outcome_contract,
    parse_notification_event as parse_kolme_notification_event_contract,
    parse_provider_block_fallback_response as parse_kolme_provider_block_fallback_response_contract,
    parse_provider_finality_receipt as parse_kolme_provider_finality_receipt,
    parse_websocket_endpoint as parse_kolme_websocket_endpoint,
    project_failed_block_txhash_receipt as project_kolme_failed_block_txhash_receipt_contract,
    project_finalized_block_txhash_receipt as project_kolme_finalized_block_txhash_receipt_contract,
    render_block_path as render_kolme_block_path,
    render_runtime_commit_wire_payload as render_kolme_runtime_commit_wire_payload_contract,
    render_signed_envelope_wire_payload as render_kolme_signed_envelope_wire_payload_contract,
    require_commit_id_matches_expected_txhash as require_kolme_commit_id_matches_expected_txhash_contract,
    require_final_receipt_finality as require_kolme_final_receipt_finality_contract,
    resolve_lookup_upper_bound as resolve_kolme_lookup_upper_bound,
    resolve_tls_ca_file_env_result as resolve_kolme_tls_ca_file_env_result_contract,
    try_take_websocket_frame as try_take_kolme_websocket_frame,
    txhash_from_commit_id as txhash_from_kolme_commit_id,
    validate_block_identity as validate_kolme_block_identity,
    validate_block_path_template as validate_kolme_block_path_template,
    validate_lookup_txhash as validate_kolme_lookup_txhash_contract,
    validate_lookup_window as validate_kolme_lookup_window,
    validate_provider_receipt_identity as validate_kolme_provider_receipt_identity_contract,
    validate_websocket_handshake_response as validate_kolme_websocket_handshake_response,
    BlockScanPolicyError, KolmeApiBroadcastRequest as KamnKolmeApiBroadcastRequest,
    KolmeApiBroadcastResponse as KamnKolmeApiBroadcastResponse,
    KolmeApiCodecError as KamnKolmeApiCodecError,
    KolmeApiNextNonceRequest as KamnKolmeApiNextNonceRequest,
    KolmeApiNextNonceResponse as KamnKolmeApiNextNonceResponse,
    KolmeCommitReceiptFinality as KamnKolmeCommitReceiptFinality,
    KolmeHttpResponsePolicyError as KamnKolmeHttpResponsePolicyError,
    KolmeHttpScheme as KamnKolmeHttpScheme, KolmeNotificationEvent as KamnKolmeNotificationEvent,
    KolmeParsedHttpEndpoint as KamnKolmeParsedHttpEndpoint,
    KolmeProviderReceiptIdentityError as KamnKolmeProviderReceiptIdentityError,
    KolmeRuntimeProviderOutcome as KamnKolmeRuntimeProviderOutcome,
    KolmeTlsPolicyError as KamnKolmeTlsPolicyError,
    KolmeTransportIoClassification as KamnKolmeTransportIoClassification,
    KolmeTransportRequestPolicyError as KamnKolmeTransportRequestPolicyError,
    KolmeWebsocketFrame as KamnKolmeWebsocketFrame,
    KolmeWebsocketPolicyError as KamnKolmeWebsocketPolicyError,
    RuntimeCommitLifecycleState as KamnKolmeRuntimeCommitLifecycleState,
    RuntimeLifecyclePolicyError as KamnKolmeRuntimeLifecyclePolicyError,
};
use std::fmt;

mod adapter_backed_client;
mod api_codec;
mod block_fallback_reconciler;
mod finality_checker;
mod fork_finality_resolver;
mod http_transport;
mod in_memory_client;
mod live_provider;
mod notifications_consumer;
mod notifications_websocket;
mod request_envelope;
mod runtime_pipeline;

/// Finality classification for a runtime commit receipt.
pub type KolmeCommitReceiptFinality = KamnKolmeCommitReceiptFinality;

/// Receipt emitted by the runtime commit client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeRuntimeCommitReceipt {
    /// Provider identifier.
    pub provider: String,
    /// Deterministic commit identifier.
    pub commit_id: String,
    /// Finality state for the receipt.
    pub finality: KolmeCommitReceiptFinality,
}

/// Typed commit submission result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeRuntimeCommitOutcome {
    /// Request was accepted and submitted.
    Submitted(KolmeRuntimeCommitReceipt),
    /// Request matched an existing idempotency key.
    Duplicate(KolmeRuntimeCommitReceipt),
    /// Request was rejected with an explicit reason.
    Rejected {
        /// Deterministic rejection reason from provider/runtime policy.
        reason: String,
    },
}

/// Runtime lifecycle state projected from commit receipt outcomes.
pub type RuntimeCommitLifecycleState = KamnKolmeRuntimeCommitLifecycleState;

pub use adapter_backed_client::AdapterBackedKolmeRuntimeCommitClient;
pub use api_codec::{
    KolmeApiBroadcastRequest, KolmeApiBroadcastResponse, KolmeApiNextNonceRequest,
    KolmeApiNextNonceResponse,
};
pub use block_fallback_reconciler::KolmeRuntimeCommitBlockFallbackReconciler;
pub use finality_checker::KolmeRuntimeCommitFinalityChecker;
pub use fork_finality_resolver::KolmeRuntimeCommitForkFinalityResolver;
pub use http_transport::KolmeRuntimeCommitHttpTransport;
pub use in_memory_client::InMemoryKolmeRuntimeCommitClient;
pub use live_provider::KolmeRuntimeCommitLiveProvider;
pub use notifications_consumer::KolmeRuntimeCommitNotificationsConsumer;
pub use notifications_websocket::{
    KolmeRuntimeCommitWebsocketConnection, KolmeRuntimeCommitWebsocketConnector,
};
pub use request_envelope::{KolmeRuntimeCommitRequest, KolmeRuntimeCommitSignedBroadcastEnvelope};
pub use runtime_pipeline::{
    RuntimeCommitFinalityProjection, RuntimeCommitLifecycleRecord, RuntimeCommitPipeline,
};

/// Abstract client interface for Kolme runtime commit submission.
pub trait KolmeRuntimeCommitClient {
    /// Submits one deterministic runtime commit request.
    fn submit_commit(
        &mut self,
        request: &KolmeRuntimeCommitRequest,
    ) -> Result<KolmeRuntimeCommitOutcome, KolmeRuntimeCommitError>;
}

/// Typed transport error class emitted when adapter-backed provider calls fail.
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

impl From<KamnKolmeTransportIoClassification> for KolmeRuntimeCommitProviderError {
    fn from(value: KamnKolmeTransportIoClassification) -> Self {
        match value {
            KamnKolmeTransportIoClassification::Timeout => Self::Timeout,
            KamnKolmeTransportIoClassification::Unavailable { reason } => {
                Self::Unavailable { reason }
            }
        }
    }
}

/// Provider receipt payload returned by adapter-facing transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeRuntimeCommitProviderReceipt {
    /// Provider identifier returned by upstream.
    pub provider: String,
    /// Commit identifier returned by upstream.
    pub commit_id: String,
    /// Receipt finality classification returned by upstream.
    pub finality: KolmeCommitReceiptFinality,
}

/// Typed notification event emitted by Kolme `/notifications` websocket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeRuntimeCommitNotificationEvent {
    /// Finalized transaction notification emitted from a new block event.
    NewBlock {
        /// Transaction hash observed in the block payload.
        txhash: String,
        /// Optional block height where the transaction finalized.
        block_height: Option<u64>,
    },
    /// Failed transaction notification emitted by processor execution path.
    FailedTransaction {
        /// Transaction hash observed in failed-transaction payload.
        txhash: String,
        /// Optional proposed block height for the failed transaction.
        proposed_height: Option<u64>,
    },
    /// Latest block watermark notification.
    LatestBlock {
        /// Latest observed block height.
        height: u64,
    },
}

impl From<KamnKolmeNotificationEvent> for KolmeRuntimeCommitNotificationEvent {
    fn from(value: KamnKolmeNotificationEvent) -> Self {
        match value {
            KamnKolmeNotificationEvent::NewBlock {
                txhash,
                block_height,
            } => Self::NewBlock {
                txhash,
                block_height,
            },
            KamnKolmeNotificationEvent::FailedTransaction {
                txhash,
                proposed_height,
            } => Self::FailedTransaction {
                txhash,
                proposed_height,
            },
            KamnKolmeNotificationEvent::LatestBlock { height } => Self::LatestBlock { height },
        }
    }
}

impl KolmeRuntimeCommitNotificationEvent {
    /// Converts notification event to a provider receipt when it carries tx finality information.
    pub fn to_provider_receipt(&self, provider: &str) -> Option<KolmeRuntimeCommitProviderReceipt> {
        let event = match self {
            Self::NewBlock {
                txhash,
                block_height,
            } => KamnKolmeNotificationEvent::NewBlock {
                txhash: txhash.clone(),
                block_height: *block_height,
            },
            Self::FailedTransaction {
                txhash,
                proposed_height,
            } => KamnKolmeNotificationEvent::FailedTransaction {
                txhash: txhash.clone(),
                proposed_height: *proposed_height,
            },
            Self::LatestBlock { height } => {
                KamnKolmeNotificationEvent::LatestBlock { height: *height }
            }
        };
        let receipt = notification_event_to_kolme_provider_receipt_contract(provider, &event)?;
        Some(KolmeRuntimeCommitProviderReceipt {
            provider: receipt.provider,
            commit_id: receipt.commit_id,
            finality: receipt.finality,
        })
    }
}

/// Provider submission outcome used by adapter-backed runtime commit clients.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeRuntimeCommitProviderOutcome {
    /// Provider accepted the submission.
    Submitted(KolmeRuntimeCommitProviderReceipt),
    /// Provider detected duplicate idempotency key.
    Duplicate(KolmeRuntimeCommitProviderReceipt),
    /// Provider rejected the submission with explicit reason.
    Rejected {
        /// Deterministic provider rejection reason.
        reason: String,
    },
}

impl From<KamnKolmeRuntimeProviderOutcome> for KolmeRuntimeCommitProviderOutcome {
    fn from(value: KamnKolmeRuntimeProviderOutcome) -> Self {
        match value {
            KamnKolmeRuntimeProviderOutcome::Submitted {
                provider,
                commit_id,
                finality,
            } => Self::Submitted(KolmeRuntimeCommitProviderReceipt {
                provider,
                commit_id,
                finality,
            }),
            KamnKolmeRuntimeProviderOutcome::Duplicate {
                provider,
                commit_id,
                finality,
            } => Self::Duplicate(KolmeRuntimeCommitProviderReceipt {
                provider,
                commit_id,
                finality,
            }),
            KamnKolmeRuntimeProviderOutcome::Rejected { reason } => Self::Rejected { reason },
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

/// Provider interface consumed by the adapter-backed runtime commit client.
pub trait KolmeRuntimeCommitProvider {
    /// Submits canonical wire payload with deterministic idempotency key.
    fn submit_runtime_commit(
        &mut self,
        wire_payload: &str,
        idempotency_key: &str,
    ) -> Result<KolmeRuntimeCommitProviderOutcome, KolmeRuntimeCommitProviderError>;
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

/// Error returned by runtime commit request validation or submission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeRuntimeCommitError {
    /// Request payload failed validation.
    InvalidRequest {
        /// Field failing validation.
        field: &'static str,
        /// Validation reason.
        reason: &'static str,
    },
    /// Operation identifier was not found in runtime pipeline state.
    UnknownOperationId {
        /// Missing operation identifier.
        operation_id: String,
    },
    /// Runtime attempted invalid lifecycle transition for receipt finality.
    InvalidFinalityTransition {
        /// Current lifecycle state label.
        from: &'static str,
        /// Target lifecycle state label.
        to: &'static str,
    },
    /// Runtime receipt field differs from the operation's existing receipt marker.
    ReceiptFieldMismatch {
        /// Field name that mismatched.
        field: &'static str,
        /// Expected persisted value.
        expected: String,
        /// Observed incoming value.
        observed: String,
    },
    /// Provider transport failed while submitting runtime commit payload.
    ProviderTransport {
        /// Typed transport error kind.
        kind: KolmeRuntimeCommitTransportErrorKind,
        /// Deterministic detail text for the transport error.
        detail: String,
    },
    /// Provider identifier did not match configured expected provider.
    ProviderMismatch {
        /// Configured provider identifier.
        expected: String,
        /// Observed provider identifier from response.
        observed: String,
    },
    /// Provider returned a non-final receipt which is rejected in adapter mode.
    NonFinalReceipt {
        /// Commit identifier returned by provider.
        commit_id: String,
        /// Observed non-final receipt state.
        finality: KolmeCommitReceiptFinality,
    },
}

impl fmt::Display for KolmeRuntimeCommitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest { field, reason } => {
                write!(f, "invalid runtime commit request {field}: {reason}")
            }
            Self::UnknownOperationId { operation_id } => {
                write!(f, "unknown runtime operation id: {operation_id}")
            }
            Self::InvalidFinalityTransition { from, to } => {
                write!(f, "invalid finality transition from {from} to {to}")
            }
            Self::ReceiptFieldMismatch {
                field,
                expected,
                observed,
            } => write!(
                f,
                "receipt field mismatch for {field}: expected '{expected}', observed '{observed}'"
            ),
            Self::ProviderTransport { kind, detail } => {
                write!(f, "provider transport failure ({kind:?}): {detail}")
            }
            Self::ProviderMismatch { expected, observed } => write!(
                f,
                "provider mismatch: expected '{expected}', observed '{observed}'"
            ),
            Self::NonFinalReceipt {
                commit_id,
                finality,
            } => write!(
                f,
                "provider receipt must be final for commit '{commit_id}', observed {}",
                commit_finality_label_contract(*finality)
            ),
        }
    }
}

impl std::error::Error for KolmeRuntimeCommitError {}

#[cfg(test)]
mod tests {
    use super::{
        classify_kolme_tls_failure_reason, KolmeRuntimeCommitError, KolmeRuntimeCommitRequest,
    };

    #[test]
    fn deterministic_request_rejects_empty_operation_id() {
        assert_eq!(
            KolmeRuntimeCommitRequest::deterministic(
                "",
                "state:abc",
                "kamn:did:agent:test-runtime",
                1,
                "payload:abc",
            ),
            Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "operation_id",
                reason: "must not be empty",
            })
        );
    }

    #[test]
    fn tls_failure_reason_classifier_detects_certificate_errors() {
        let reason = classify_kolme_tls_failure_reason(
            "verify error:num=18:self-signed certificate\ncertificate verify failed",
        );
        assert_eq!(reason, "tls certificate verification failed");
    }

    #[test]
    fn tls_failure_reason_classifier_detects_handshake_errors() {
        let reason =
            classify_kolme_tls_failure_reason("ssl routines:ssl3_get_record:wrong version number");
        assert_eq!(reason, "tls handshake failed");
    }
}
