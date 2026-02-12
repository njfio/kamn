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
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

mod in_memory_client;
mod notifications_websocket;
mod runtime_pipeline;

/// Runtime commit submission request for the Kolme execution path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeRuntimeCommitRequest {
    /// Deterministic operation identifier.
    pub operation_id: String,
    /// Runtime state root/hash reference.
    pub state_root: String,
    /// Actor DID submitting the runtime commit.
    pub actor_did: AgentDid,
    /// Monotonic submission nonce.
    pub nonce: u64,
    /// Deterministic payload hash marker.
    pub payload_hash: String,
    idempotency_key: String,
}

impl KolmeRuntimeCommitRequest {
    /// Builds a deterministic commit request and validates required invariants.
    pub fn deterministic(
        operation_id: &str,
        state_root: &str,
        actor_did: &str,
        nonce: u64,
        payload_hash: &str,
    ) -> Result<Self, KolmeRuntimeCommitError> {
        let actor_did =
            AgentDid::parse(actor_did).map_err(|_| KolmeRuntimeCommitError::InvalidRequest {
                field: "actor_did",
                reason: "must be a valid KAMN DID",
            })?;
        let (operation_id, state_root, payload_hash) =
            normalize_kolme_runtime_commit_request_fields_contract(
                operation_id,
                state_root,
                payload_hash,
            );
        let actor_did_value = actor_did.as_str().to_owned();
        let idempotency_key = deterministic_runtime_commit_idempotency_key_contract(
            operation_id.as_str(),
            state_root.as_str(),
            actor_did_value.as_str(),
            nonce,
            payload_hash.as_str(),
        );

        let request = Self {
            operation_id,
            state_root,
            actor_did,
            nonce,
            payload_hash,
            idempotency_key,
        };
        request.validate()?;
        Ok(request)
    }

    /// Returns deterministic request payload in canonical field order.
    pub fn to_wire_payload(&self) -> String {
        render_kolme_runtime_commit_wire_payload_contract(
            self.operation_id.as_str(),
            self.state_root.as_str(),
            self.actor_did.as_str(),
            self.nonce,
            self.payload_hash.as_str(),
            self.idempotency_key.as_str(),
        )
    }

    /// Returns the deterministic idempotency key.
    pub fn idempotency_key(&self) -> &str {
        &self.idempotency_key
    }

    /// Translates a canonical runtime commit into a signed broadcast envelope.
    pub fn translate_to_signed_broadcast_envelope(
        &self,
        signer_key_id: &str,
        signed_message: &str,
        signature: &str,
        recovery_id: u8,
    ) -> Result<KolmeRuntimeCommitSignedBroadcastEnvelope, KolmeRuntimeCommitError> {
        self.validate()?;
        let canonical_message = self.to_wire_payload();
        if !is_kolme_canonical_runtime_commit_signed_message_contract(
            canonical_message.as_str(),
            signed_message,
        ) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "signed_message",
                reason: "must match canonical runtime commit wire payload",
            });
        }
        KolmeRuntimeCommitSignedBroadcastEnvelope::new(
            signer_key_id,
            signed_message,
            signature,
            recovery_id,
        )
    }

    /// Validates commit request schema and invariant boundaries.
    pub fn validate(&self) -> Result<(), KolmeRuntimeCommitError> {
        if !is_kolme_valid_runtime_operation_id_input_contract(self.operation_id.as_str()) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "operation_id",
                reason: "must not be empty",
            });
        }
        if !is_kolme_valid_runtime_state_root_input_contract(self.state_root.as_str()) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "state_root",
                reason: "must not be empty",
            });
        }
        if !is_kolme_valid_runtime_nonce_input_contract(self.nonce) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "nonce",
                reason: "must be positive",
            });
        }
        if !is_kolme_valid_runtime_payload_hash_input_contract(self.payload_hash.as_str()) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "payload_hash",
                reason: "must not be empty",
            });
        }
        if !are_kolme_runtime_commit_request_fields_single_line_contract(
            self.operation_id.as_str(),
            self.state_root.as_str(),
            self.payload_hash.as_str(),
        ) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "wire_payload",
                reason: "fields must be single-line",
            });
        }
        Ok(())
    }
}

/// Signed envelope that binds canonical runtime commit message to signing identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeRuntimeCommitSignedBroadcastEnvelope {
    /// Signer key identifier used by the external custody boundary.
    pub signer_key_id: String,
    /// Canonical runtime commit message that was signed.
    pub message: String,
    /// Signature bytes/encoding for the message.
    pub signature: String,
    /// Signature recovery identifier.
    pub recovery_id: u8,
}

impl KolmeRuntimeCommitSignedBroadcastEnvelope {
    /// Builds a signed broadcast envelope with deterministic validation.
    pub fn new(
        signer_key_id: &str,
        message: &str,
        signature: &str,
        recovery_id: u8,
    ) -> Result<Self, KolmeRuntimeCommitError> {
        if !is_kolme_valid_signed_envelope_signer_key_id_input_contract(signer_key_id) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "signer_key_id",
                reason: "must not be empty",
            });
        }
        if !is_kolme_valid_signed_envelope_message_input_contract(message) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "signed_message",
                reason: "must not be empty",
            });
        }
        if !is_kolme_valid_signed_envelope_signature_input_contract(signature) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "signature",
                reason: "must not be empty",
            });
        }
        let (signer_key_id, message, signature) =
            normalize_kolme_runtime_commit_signed_envelope_fields_contract(
                signer_key_id,
                message,
                signature,
            );
        Ok(Self {
            signer_key_id,
            message,
            signature,
            recovery_id,
        })
    }

    /// Returns canonical wire payload used by fork submit profile before normalization.
    pub fn to_wire_payload(&self) -> String {
        render_kolme_signed_envelope_wire_payload_contract(
            self.signer_key_id.as_str(),
            self.message.as_str(),
            self.signature.as_str(),
            self.recovery_id,
        )
    }

    /// Converts the envelope into a Kolme `/broadcast` request payload.
    pub fn to_broadcast_request(
        &self,
    ) -> Result<KolmeApiBroadcastRequest, KolmeRuntimeCommitError> {
        KolmeApiBroadcastRequest::new(
            self.message.as_str(),
            self.signature.as_str(),
            self.recovery_id,
        )
    }
}

/// Typed nonce lookup request for Kolme `/get-next-nonce`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeApiNextNonceRequest {
    /// Public key used to resolve next nonce and account identity.
    pub pubkey: String,
}

impl KolmeApiNextNonceRequest {
    /// Builds a deterministic nonce lookup request.
    pub fn new(pubkey: &str) -> Result<Self, KolmeRuntimeCommitError> {
        let extracted = KamnKolmeApiNextNonceRequest::new(pubkey).map_err(|error| match error {
            KamnKolmeApiCodecError::InvalidRequest { field, reason } => {
                KolmeRuntimeCommitError::InvalidRequest { field, reason }
            }
            KamnKolmeApiCodecError::MalformedResponse { .. } => {
                KolmeRuntimeCommitError::InvalidRequest {
                    field: "codec_payload",
                    reason: "must be valid json",
                }
            }
        })?;
        Ok(Self {
            pubkey: extracted.pubkey,
        })
    }

    /// Returns encoded request path for the configured nonce endpoint.
    pub fn query_path(&self, nonce_path: &str) -> String {
        KamnKolmeApiNextNonceRequest {
            pubkey: self.pubkey.clone(),
        }
        .query_path(nonce_path)
    }
}

/// Typed nonce lookup response for Kolme `/get-next-nonce`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeApiNextNonceResponse {
    /// Monotonic next nonce for the provided public key.
    pub next_nonce: u64,
    /// Optional account identifier mapped to the provided public key.
    pub account_id: Option<String>,
}

impl KolmeApiNextNonceResponse {
    /// Parses one nonce lookup response JSON payload.
    pub fn parse_json(response: &str) -> Result<Self, KolmeRuntimeCommitProviderError> {
        let extracted =
            KamnKolmeApiNextNonceResponse::parse_json(response).map_err(|error| match error {
                KamnKolmeApiCodecError::InvalidRequest { field, reason } => {
                    KolmeRuntimeCommitProviderError::MalformedResponse {
                        reason: format!("invalid request {field}: {reason}"),
                    }
                }
                KamnKolmeApiCodecError::MalformedResponse { reason } => {
                    KolmeRuntimeCommitProviderError::MalformedResponse { reason }
                }
            })?;
        Ok(Self {
            next_nonce: extracted.next_nonce,
            account_id: extracted.account_id,
        })
    }
}

/// Typed broadcast request payload for Kolme `/broadcast`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeApiBroadcastRequest {
    /// Tagged transaction message payload.
    pub message: String,
    /// Chain signature for the transaction message payload.
    pub signature: String,
    /// Signature recovery identifier.
    pub recovery_id: u8,
}

impl KolmeApiBroadcastRequest {
    /// Builds a deterministic broadcast request payload.
    pub fn new(
        message: &str,
        signature: &str,
        recovery_id: u8,
    ) -> Result<Self, KolmeRuntimeCommitError> {
        let extracted = KamnKolmeApiBroadcastRequest::new(message, signature, recovery_id)
            .map_err(|error| match error {
                KamnKolmeApiCodecError::InvalidRequest { field, reason } => {
                    KolmeRuntimeCommitError::InvalidRequest { field, reason }
                }
                KamnKolmeApiCodecError::MalformedResponse { .. } => {
                    KolmeRuntimeCommitError::InvalidRequest {
                        field: "codec_payload",
                        reason: "must be valid json",
                    }
                }
            })?;
        Ok(Self {
            message: extracted.message,
            signature: extracted.signature,
            recovery_id: extracted.recovery_id,
        })
    }

    /// Returns deterministic JSON payload in canonical field order.
    pub fn to_json_payload(&self) -> String {
        KamnKolmeApiBroadcastRequest {
            message: self.message.clone(),
            signature: self.signature.clone(),
            recovery_id: self.recovery_id,
        }
        .to_json_payload()
    }
}

/// Typed broadcast response payload for Kolme `/broadcast`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeApiBroadcastResponse {
    /// Transaction hash identifier from broadcast response.
    pub txhash: String,
}

impl KolmeApiBroadcastResponse {
    /// Parses one broadcast response JSON payload.
    pub fn parse_json(response: &str) -> Result<Self, KolmeRuntimeCommitProviderError> {
        let extracted =
            KamnKolmeApiBroadcastResponse::parse_json(response).map_err(|error| match error {
                KamnKolmeApiCodecError::InvalidRequest { field, reason } => {
                    KolmeRuntimeCommitProviderError::MalformedResponse {
                        reason: format!("invalid request {field}: {reason}"),
                    }
                }
                KamnKolmeApiCodecError::MalformedResponse { reason } => {
                    KolmeRuntimeCommitProviderError::MalformedResponse { reason }
                }
            })?;
        Ok(Self {
            txhash: extracted.txhash,
        })
    }
}

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

pub use in_memory_client::InMemoryKolmeRuntimeCommitClient;
pub use notifications_websocket::{
    KolmeRuntimeCommitWebsocketConnection, KolmeRuntimeCommitWebsocketConnector,
};
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

type ParsedHttpEndpoint = KamnKolmeParsedHttpEndpoint;

type HttpScheme = KamnKolmeHttpScheme;

/// Dependency-free HTTP transport implementation for runtime commit submit/finality calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeRuntimeCommitHttpTransport {
    timeout_seconds: u64,
    authorization_header: Option<String>,
}

impl KolmeRuntimeCommitHttpTransport {
    /// Builds a concrete HTTP transport with deterministic timeout validation.
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

    /// Builds a concrete HTTP transport with deterministic authorization header configuration.
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

    /// Fetches one typed nonce response from `/get-next-nonce`.
    pub fn fetch_next_nonce(
        &mut self,
        base_url: &str,
        nonce_path: &str,
        request: &KolmeApiNextNonceRequest,
    ) -> Result<KolmeApiNextNonceResponse, KolmeRuntimeCommitProviderError> {
        let path = request.query_path(nonce_path);
        let response = self.execute_request(base_url, path.as_str(), "GET", None, &[])?;
        KolmeApiNextNonceResponse::parse_json(response.as_str())
    }

    /// Submits one typed broadcast request to `/broadcast`.
    pub fn submit_broadcast_request(
        &mut self,
        base_url: &str,
        submit_path: &str,
        request: &KolmeApiBroadcastRequest,
        idempotency_key: &str,
    ) -> Result<KolmeApiBroadcastResponse, KolmeRuntimeCommitProviderError> {
        if !is_kolme_valid_transport_idempotency_key_input_contract(idempotency_key) {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "idempotency_key must not be empty".to_owned(),
            });
        }
        let idempotency_key =
            normalize_kolme_transport_idempotency_key_input_contract(idempotency_key);
        let submit_path = normalize_kolme_broadcast_submit_path_input_contract(submit_path);
        let payload = request.to_json_payload();
        let response = self.execute_request(
            base_url,
            submit_path,
            "PUT",
            Some(payload.as_str()),
            &[
                ("Content-Type", "application/json"),
                ("X-Idempotency-Key", idempotency_key),
            ],
        )?;
        KolmeApiBroadcastResponse::parse_json(response.as_str())
    }

    fn execute_request(
        &self,
        base_url: &str,
        path: &str,
        method: &str,
        body: Option<&str>,
        headers: &[(&str, &str)],
    ) -> Result<String, KolmeRuntimeCommitProviderError> {
        let endpoint = parse_kolme_http_endpoint(base_url, path).map_err(|error| {
            KolmeRuntimeCommitProviderError::Unavailable {
                reason: error.to_string(),
            }
        })?;
        let payload = body.unwrap_or("");
        let mut request = format!(
            "{method} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n",
            endpoint.target_path, endpoint.host_header
        );
        for (header_name, header_value) in headers {
            request.push_str(header_name);
            request.push_str(": ");
            request.push_str(header_value);
            request.push_str("\r\n");
        }
        if let Some(authorization_header) = self.authorization_header.as_deref() {
            request.push_str("Authorization: ");
            request.push_str(authorization_header);
            request.push_str("\r\n");
        }
        if body.is_some() {
            request.push_str(format!("Content-Length: {}\r\n", payload.len()).as_str());
        }
        request.push_str("\r\n");
        if body.is_some() {
            request.push_str(payload);
        }

        let response_bytes = match endpoint.scheme {
            HttpScheme::Http => self.execute_http_request(endpoint, request.as_bytes())?,
            HttpScheme::Https => self.execute_https_request(endpoint, request.as_bytes())?,
        };
        parse_kolme_http_response_body(response_bytes).map_err(|error| match error {
            KamnKolmeHttpResponsePolicyError::Timeout => KolmeRuntimeCommitProviderError::Timeout,
            KamnKolmeHttpResponsePolicyError::Unavailable { reason } => {
                KolmeRuntimeCommitProviderError::Unavailable { reason }
            }
            KamnKolmeHttpResponsePolicyError::Malformed { reason } => {
                KolmeRuntimeCommitProviderError::MalformedResponse { reason }
            }
        })
    }

    fn execute_http_request(
        &self,
        endpoint: ParsedHttpEndpoint,
        request: &[u8],
    ) -> Result<Vec<u8>, KolmeRuntimeCommitProviderError> {
        let mut stream =
            TcpStream::connect((endpoint.host.as_str(), endpoint.port)).map_err(|error| {
                KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
            })?;
        let timeout = Duration::from_secs(self.timeout_seconds);
        stream.set_read_timeout(Some(timeout)).map_err(|error| {
            KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
        })?;
        stream.set_write_timeout(Some(timeout)).map_err(|error| {
            KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
        })?;
        stream.write_all(request).map_err(|error| {
            KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
        })?;

        let mut response_bytes = Vec::new();
        stream.read_to_end(&mut response_bytes).map_err(|error| {
            KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
        })?;
        Ok(response_bytes)
    }

    fn execute_https_request(
        &self,
        endpoint: ParsedHttpEndpoint,
        request: &[u8],
    ) -> Result<Vec<u8>, KolmeRuntimeCommitProviderError> {
        let connect_target = format!("{}:{}", endpoint.host, endpoint.port);
        let mut command = Command::new("openssl");
        command
            .arg("s_client")
            .arg("-quiet")
            .arg("-verify_return_error")
            .arg("-servername")
            .arg(endpoint.host.as_str())
            .arg("-connect")
            .arg(connect_target.as_str())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let configured_ca_file =
            resolve_kolme_tls_ca_file_env_result_contract(std::env::var("KAMN_KOLME_TLS_CA_FILE"))
                .map_err(|error| match error {
                    KamnKolmeTlsPolicyError::Unavailable { reason } => {
                        KolmeRuntimeCommitProviderError::Unavailable { reason }
                    }
                })?;
        if let Some(ca_file) = configured_ca_file {
            command.arg("-CAfile").arg(ca_file);
        }

        let mut child =
            command
                .spawn()
                .map_err(|error| KolmeRuntimeCommitProviderError::Unavailable {
                    reason: format!("tls command spawn failed: {error}"),
                })?;
        {
            let mut stdin =
                child
                    .stdin
                    .take()
                    .ok_or_else(|| KolmeRuntimeCommitProviderError::Unavailable {
                        reason: "tls command stdin unavailable".to_owned(),
                    })?;
            stdin.write_all(request).map_err(|error| {
                KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
            })?;
        }

        let timeout = Duration::from_secs(self.timeout_seconds);
        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {
                    if started.elapsed() >= timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(KolmeRuntimeCommitProviderError::Timeout);
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => {
                    return Err(KolmeRuntimeCommitProviderError::Unavailable {
                        reason: format!("tls command wait failed: {error}"),
                    });
                }
            }
        }

        let output = child.wait_with_output().map_err(|error| {
            KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
        })?;
        let looks_like_http_response = output.stdout.starts_with(b"HTTP/1.")
            && output.stdout.windows(4).any(|window| window == b"\r\n\r\n");
        if !output.status.success() && !looks_like_http_response {
            return Err(KolmeRuntimeCommitProviderError::Unavailable {
                reason: classify_kolme_tls_failure_reason(
                    String::from_utf8_lossy(&output.stderr).as_ref(),
                ),
            });
        }
        if !is_kolme_valid_http_response_bytes_input_contract(output.stdout.as_slice()) {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "tls response body is empty".to_owned(),
            });
        }
        Ok(output.stdout)
    }
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

impl KolmeRuntimeCommitProviderTransport for KolmeRuntimeCommitHttpTransport {
    fn submit_runtime_commit(
        &mut self,
        base_url: &str,
        submit_path: &str,
        wire_payload: &str,
        idempotency_key: &str,
    ) -> Result<String, KolmeRuntimeCommitProviderError> {
        if !is_kolme_valid_transport_wire_payload_input_contract(wire_payload) {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "wire_payload must not be empty".to_owned(),
            });
        }
        if !is_kolme_valid_transport_idempotency_key_input_contract(idempotency_key) {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "idempotency_key must not be empty".to_owned(),
            });
        }
        if is_kolme_broadcast_submit_path_contract(submit_path) {
            let payload = normalize_kolme_broadcast_payload_contract(wire_payload, idempotency_key)
                .map_err(|error| KolmeRuntimeCommitProviderError::MalformedResponse {
                    reason: error.to_string(),
                })?;
            return self.execute_request(
                base_url,
                submit_path,
                "PUT",
                Some(payload.as_str()),
                &[
                    ("Content-Type", "application/json"),
                    ("X-Idempotency-Key", idempotency_key),
                ],
            );
        }
        self.execute_request(
            base_url,
            submit_path,
            "POST",
            Some(wire_payload),
            &[
                ("Content-Type", "text/plain"),
                ("X-Idempotency-Key", idempotency_key),
            ],
        )
    }
}

impl KolmeRuntimeCommitFinalityTransport for KolmeRuntimeCommitHttpTransport {
    fn fetch_runtime_commit_finality(
        &mut self,
        base_url: &str,
        status_path: &str,
        commit_id: &str,
    ) -> Result<String, KolmeRuntimeCommitProviderError> {
        let path = compose_kolme_finality_status_path(status_path, commit_id).map_err(|error| {
            KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: error.to_string(),
            }
        })?;
        self.execute_request(base_url, path.as_str(), "GET", None, &[])
    }
}

impl KolmeRuntimeCommitBlockFallbackTransport for KolmeRuntimeCommitHttpTransport {
    fn fetch_block_by_height(
        &mut self,
        base_url: &str,
        block_path_template: &str,
        height: u64,
    ) -> Result<String, KolmeRuntimeCommitProviderError> {
        if !is_kolme_valid_block_lookup_height_contract(height) {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "block height must be positive".to_owned(),
            });
        }
        let block_path = render_kolme_block_path(block_path_template, height).map_err(|error| {
            KolmeRuntimeCommitProviderError::Unavailable {
                reason: error.to_string(),
            }
        })?;
        self.execute_request(base_url, block_path.as_str(), "GET", None, &[])
    }
}

/// Deterministic finality checker for live backend runtime commit receipts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeRuntimeCommitFinalityChecker<T> {
    base_url: String,
    status_path: String,
    transport: T,
}

impl<T: KolmeRuntimeCommitFinalityTransport> KolmeRuntimeCommitFinalityChecker<T> {
    /// Builds a finality checker with deterministic endpoint validation.
    pub fn new(
        base_url: &str,
        status_path: &str,
        transport: T,
    ) -> Result<Self, KolmeRuntimeCommitError> {
        if !is_kolme_valid_finality_base_url_input_contract(base_url) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider_base_url",
                reason: "must not be empty",
            });
        }
        if !is_kolme_valid_finality_status_path_input_contract(status_path) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider_status_path",
                reason: "must not be empty",
            });
        }
        let (base_url, status_path) =
            normalize_kolme_finality_endpoint_inputs_contract(base_url, status_path);
        Ok(Self {
            base_url,
            status_path,
            transport,
        })
    }

    /// Fetches and parses one backend finality response for the provided commit.
    pub fn check_commit_finality(
        &mut self,
        commit_id: &str,
    ) -> Result<KolmeRuntimeCommitProviderReceipt, KolmeRuntimeCommitProviderError> {
        if !is_kolme_valid_runtime_commit_id_request_contract(commit_id) {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "commit_id must not be empty".to_owned(),
            });
        }

        let response = self.transport.fetch_runtime_commit_finality(
            self.base_url.as_str(),
            self.status_path.as_str(),
            commit_id,
        )?;
        let receipt = parse_kolme_provider_finality_receipt(response.as_str(), commit_id).map_err(
            |error| KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: error.to_string(),
            },
        )?;
        Ok(KolmeRuntimeCommitProviderReceipt {
            provider: receipt.provider,
            commit_id: receipt.commit_id,
            finality: receipt.finality,
        })
    }

    /// Polls backend finality and returns the first non-pending receipt.
    pub fn poll_finality(
        &mut self,
        commit_id: &str,
        max_attempts: u32,
    ) -> Result<KolmeRuntimeCommitProviderReceipt, KolmeRuntimeCommitProviderError> {
        if !is_kolme_valid_poll_attempt_budget_contract(max_attempts) {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "max_attempts must be positive".to_owned(),
            });
        }
        for _ in 0..max_attempts {
            let receipt = self.check_commit_finality(commit_id)?;
            if is_kolme_terminal_receipt_finality_contract(receipt.finality) {
                return Ok(receipt);
            }
        }
        Err(KolmeRuntimeCommitProviderError::Timeout)
    }
}

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

/// Deterministic notifications consumer for Kolme websocket events with bounded reconnect policy.
pub struct KolmeRuntimeCommitNotificationsConsumer<C>
where
    C: KolmeRuntimeCommitNotificationsConnector,
{
    notifications_url: String,
    provider: String,
    max_reconnect_attempts: u32,
    connector: C,
    connection: Option<C::Connection>,
}

impl<C> KolmeRuntimeCommitNotificationsConsumer<C>
where
    C: KolmeRuntimeCommitNotificationsConnector,
{
    /// Builds notifications consumer from HTTP base URL and notifications path.
    pub fn new(
        base_url: &str,
        notifications_path: &str,
        provider: &str,
        max_reconnect_attempts: u32,
        connector: C,
    ) -> Result<Self, KolmeRuntimeCommitError> {
        if !is_kolme_valid_notifications_provider_input_contract(provider) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "notifications_provider",
                reason: "must not be empty",
            });
        }
        if !is_kolme_valid_notifications_reconnect_budget_contract(max_reconnect_attempts) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "notifications_max_reconnect_attempts",
                reason: "must be positive",
            });
        }
        let notifications_url =
            compose_kolme_notifications_websocket_url(base_url, notifications_path)
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
        Ok(Self {
            notifications_url,
            provider: normalize_kolme_notifications_provider_input_contract(provider).to_owned(),
            max_reconnect_attempts,
            connector,
            connection: None,
        })
    }

    /// Reads and parses one notifications event, reconnecting when the stream drops.
    pub fn next_notification_event(
        &mut self,
    ) -> Result<KolmeRuntimeCommitNotificationEvent, KolmeRuntimeCommitProviderError> {
        let mut reconnect_attempts = 0_u32;

        loop {
            if self.connection.is_none() {
                match self.connector.connect(self.notifications_url.as_str()) {
                    Ok(connection) => self.connection = Some(connection),
                    Err(_) => {
                        reconnect_attempts += 1;
                        if reconnect_attempts >= self.max_reconnect_attempts {
                            return Err(KolmeRuntimeCommitProviderError::Unavailable {
                                reason:
                                    compose_kolme_notifications_reconnect_exhausted_reason_contract(
                                        self.max_reconnect_attempts,
                                    ),
                            });
                        }
                        continue;
                    }
                }
            }

            let result = self
                .connection
                .as_mut()
                .expect("connection should exist before read")
                .read_text_message();
            match result {
                Ok(Some(payload)) => {
                    let event = parse_kolme_notification_event_contract(payload.as_str()).map_err(
                        |error| KolmeRuntimeCommitProviderError::MalformedResponse {
                            reason: error.to_string(),
                        },
                    )?;
                    return Ok(event.into());
                }
                Ok(None) | Err(_) => {
                    self.connection = None;
                    reconnect_attempts += 1;
                    if reconnect_attempts >= self.max_reconnect_attempts {
                        return Err(KolmeRuntimeCommitProviderError::Unavailable {
                            reason: compose_kolme_notifications_reconnect_exhausted_reason_contract(
                                self.max_reconnect_attempts,
                            ),
                        });
                    }
                }
            }
        }
    }

    /// Reads notification events until one can be mapped to a commit receipt.
    pub fn next_commit_receipt(
        &mut self,
    ) -> Result<KolmeRuntimeCommitProviderReceipt, KolmeRuntimeCommitProviderError> {
        loop {
            let event = self.next_notification_event()?;
            if let Some(receipt) = event.to_provider_receipt(self.provider.as_str()) {
                return Ok(receipt);
            }
        }
    }
}

/// Fork-profile finality resolver that composes notifications and block fallback lookups.
pub struct KolmeRuntimeCommitForkFinalityResolver<C, T>
where
    C: KolmeRuntimeCommitNotificationsConnector,
    T: KolmeRuntimeCommitBlockFallbackTransport,
{
    notifications_consumer: KolmeRuntimeCommitNotificationsConsumer<C>,
    block_fallback_reconciler: KolmeRuntimeCommitBlockFallbackReconciler<T>,
}

impl<C, T> KolmeRuntimeCommitForkFinalityResolver<C, T>
where
    C: KolmeRuntimeCommitNotificationsConnector,
    T: KolmeRuntimeCommitBlockFallbackTransport,
{
    /// Builds a fork finality resolver from notifications and block fallback components.
    pub fn new(
        notifications_consumer: KolmeRuntimeCommitNotificationsConsumer<C>,
        block_fallback_reconciler: KolmeRuntimeCommitBlockFallbackReconciler<T>,
    ) -> Self {
        Self {
            notifications_consumer,
            block_fallback_reconciler,
        }
    }

    /// Resolves finality for one commit id using notifications first, then bounded block fallback.
    pub fn resolve_commit_finality(
        &mut self,
        commit_id: &str,
        from_height: u64,
        latest_height: u64,
    ) -> Result<KolmeRuntimeCommitProviderReceipt, KolmeRuntimeCommitProviderError> {
        let expected_txhash = txhash_from_kolme_commit_id(commit_id).map_err(|error| {
            KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: error.to_string(),
            }
        })?;

        match self.notifications_consumer.next_notification_event() {
            Ok(event) => match event {
                KolmeRuntimeCommitNotificationEvent::LatestBlock { height } => {
                    let upper_bound =
                        resolve_kolme_lookup_upper_bound(from_height, latest_height, height);
                    self.block_fallback_reconciler.reconcile_txhash(
                        expected_txhash.as_str(),
                        from_height,
                        upper_bound,
                    )
                }
                _ => {
                    let receipt = event
                        .to_provider_receipt(self.notifications_consumer.provider.as_str())
                        .ok_or_else(|| KolmeRuntimeCommitProviderError::MalformedResponse {
                            reason: "notification event did not carry receipt data".to_owned(),
                        })?;
                    require_kolme_commit_id_matches_expected_txhash_contract(
                        receipt.commit_id.as_str(),
                        expected_txhash.as_str(),
                    )
                    .map_err(|error| {
                        KolmeRuntimeCommitProviderError::MalformedResponse {
                            reason: error.to_string(),
                        }
                    })?;
                    Ok(receipt)
                }
            },
            Err(KolmeRuntimeCommitProviderError::Unavailable { .. })
            | Err(KolmeRuntimeCommitProviderError::Timeout) => self
                .block_fallback_reconciler
                .reconcile_txhash(expected_txhash.as_str(), from_height, latest_height),
            Err(error @ KolmeRuntimeCommitProviderError::MalformedResponse { .. }) => Err(error),
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
