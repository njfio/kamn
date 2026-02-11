//! Deterministic runtime-commit request/receipt contracts for Kolme integration.

use crate::AgentDid;
use kamn_kolme::{
    classify_tls_failure_reason as classify_kolme_tls_failure_reason,
    classify_transport_io_error as classify_kolme_transport_io_error,
    commit_finality_from_receipt_finality as commit_finality_from_receipt_finality_contract,
    commit_finality_label as commit_finality_label_contract,
    compose_finality_status_path as compose_kolme_finality_status_path,
    compose_notifications_websocket_url as compose_kolme_notifications_websocket_url,
    deterministic_backend_commit_id as deterministic_kolme_backend_commit_id,
    deterministic_runtime_commit_id as deterministic_runtime_commit_id_contract,
    deterministic_runtime_commit_idempotency_key as deterministic_runtime_commit_idempotency_key_contract,
    escape_json_string as escape_kolme_json_string,
    find_http_header_boundary as find_kolme_http_header_boundary,
    is_broadcast_submit_path as is_kolme_broadcast_submit_path_contract,
    lifecycle_state_for_finality as lifecycle_state_for_finality_contract,
    lifecycle_state_label as lifecycle_state_label_contract,
    normalize_broadcast_payload as normalize_kolme_broadcast_payload_contract,
    parse_authorization_header_value as parse_kolme_authorization_header_value,
    parse_block_fallback_response as parse_kolme_block_fallback_response_contract,
    parse_commit_id_from_response_fields as parse_kolme_commit_id_from_response_fields,
    parse_commit_receipt_finality as parse_kolme_commit_receipt_finality,
    parse_fork_block_fallback_response as parse_kolme_fork_block_fallback_response_contract,
    parse_http_endpoint as parse_kolme_http_endpoint,
    parse_http_response_body as parse_kolme_http_response_body,
    parse_live_provider_outcome as parse_kolme_live_provider_outcome,
    parse_notification_event as parse_kolme_notification_event_contract,
    parse_provider_response_fields as parse_kolme_provider_response_fields,
    parse_tls_ca_file_env_value as parse_kolme_tls_ca_file_env_value,
    parse_websocket_endpoint as parse_kolme_websocket_endpoint,
    render_block_path as render_kolme_block_path,
    required_provider_response_field as required_kolme_provider_response_field,
    try_take_websocket_frame as try_take_kolme_websocket_frame,
    txhash_from_commit_id as txhash_from_kolme_commit_id,
    validate_block_identity as validate_kolme_block_identity,
    validate_block_path_template as validate_kolme_block_path_template,
    validate_lookup_window as validate_kolme_lookup_window,
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
    KolmeProviderOutcome as KamnKolmeProviderOutcome,
    KolmeProviderOutcomePolicyError as KamnKolmeProviderOutcomePolicyError,
    KolmeTlsPolicyError as KamnKolmeTlsPolicyError,
    KolmeTransportIoClassification as KamnKolmeTransportIoClassification,
    KolmeTransportRequestPolicyError as KamnKolmeTransportRequestPolicyError,
    KolmeWebsocketFrame as KamnKolmeWebsocketFrame,
    KolmeWebsocketPolicyError as KamnKolmeWebsocketPolicyError,
    RuntimeCommitLifecycleState as KamnKolmeRuntimeCommitLifecycleState,
};
use std::collections::HashMap;
use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

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
        let actor_did_value = actor_did.as_str().to_owned();
        let idempotency_key = deterministic_runtime_commit_idempotency_key_contract(
            operation_id,
            state_root,
            actor_did_value.as_str(),
            nonce,
            payload_hash,
        );

        let request = Self {
            operation_id: operation_id.trim().to_owned(),
            state_root: state_root.trim().to_owned(),
            actor_did,
            nonce,
            payload_hash: payload_hash.trim().to_owned(),
            idempotency_key,
        };
        request.validate()?;
        Ok(request)
    }

    /// Returns deterministic request payload in canonical field order.
    pub fn to_wire_payload(&self) -> String {
        format!(
            "operation_id={}\nstate_root={}\nactor_did={}\nnonce={}\npayload_hash={}\nidempotency_key={}\n",
            self.operation_id,
            self.state_root,
            self.actor_did.as_str(),
            self.nonce,
            self.payload_hash,
            self.idempotency_key
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
        if signed_message != canonical_message {
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
        if self.operation_id.trim().is_empty() {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "operation_id",
                reason: "must not be empty",
            });
        }
        if self.state_root.trim().is_empty() {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "state_root",
                reason: "must not be empty",
            });
        }
        if self.nonce == 0 {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "nonce",
                reason: "must be positive",
            });
        }
        if self.payload_hash.trim().is_empty() {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "payload_hash",
                reason: "must not be empty",
            });
        }
        if self.operation_id.contains('\n')
            || self.state_root.contains('\n')
            || self.payload_hash.contains('\n')
        {
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
        let signer_key_id = signer_key_id.trim();
        if signer_key_id.is_empty() {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "signer_key_id",
                reason: "must not be empty",
            });
        }
        let message = message.trim();
        if message.is_empty() {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "signed_message",
                reason: "must not be empty",
            });
        }
        let signature = signature.trim();
        if signature.is_empty() {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "signature",
                reason: "must not be empty",
            });
        }
        Ok(Self {
            signer_key_id: signer_key_id.to_owned(),
            message: message.to_owned(),
            signature: signature.to_owned(),
            recovery_id,
        })
    }

    /// Returns canonical wire payload used by fork submit profile before normalization.
    pub fn to_wire_payload(&self) -> String {
        format!(
            "{{\"signer_key_id\":\"{}\",\"message\":\"{}\",\"signature\":\"{}\",\"recovery_id\":{}}}",
            escape_kolme_json_string(self.signer_key_id.as_str()),
            escape_kolme_json_string(self.message.as_str()),
            escape_kolme_json_string(self.signature.as_str()),
            self.recovery_id
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

/// One runtime operation lifecycle record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCommitLifecycleRecord {
    /// Runtime operation identifier.
    pub operation_id: String,
    /// Deterministic idempotency key for the operation.
    pub idempotency_key: String,
    /// Projected lifecycle state.
    pub state: RuntimeCommitLifecycleState,
    /// Whether runtime should requeue/retry polling for this operation.
    pub needs_requeue: bool,
    /// Last known receipt provider marker.
    pub receipt_provider: Option<String>,
    /// Last known receipt identifier.
    pub receipt_commit_id: Option<String>,
    /// Last known rejection/failure reason.
    pub last_error_reason: Option<String>,
}

/// Projection summary for runtime commit lifecycle counts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeCommitFinalityProjection {
    /// Number of pending operations.
    pub pending_count: usize,
    /// Number of finalized operations.
    pub final_count: usize,
    /// Number of failed operations.
    pub failed_count: usize,
}

/// Deterministic runtime pipeline for commit receipt confirmation and finality projection.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeCommitPipeline {
    records_by_operation_id: HashMap<String, RuntimeCommitLifecycleRecord>,
}

impl RuntimeCommitPipeline {
    /// Constructs an empty runtime commit pipeline.
    pub fn new() -> Self {
        Self::default()
    }

    /// Submits one runtime commit through the provided commit client and records lifecycle state.
    pub fn submit_with_client<C: KolmeRuntimeCommitClient>(
        &mut self,
        client: &mut C,
        request: KolmeRuntimeCommitRequest,
    ) -> Result<RuntimeCommitLifecycleRecord, KolmeRuntimeCommitError> {
        let outcome = client.submit_commit(&request)?;
        let record = lifecycle_record_from_outcome(&request, &outcome);
        self.records_by_operation_id
            .insert(request.operation_id.clone(), record.clone());
        Ok(record)
    }

    /// Applies explicit receipt finality update for an existing operation.
    pub fn apply_receipt_finality(
        &mut self,
        operation_id: &str,
        finality: KolmeCommitReceiptFinality,
        receipt_provider: &str,
        receipt_commit_id: &str,
    ) -> Result<RuntimeCommitLifecycleRecord, KolmeRuntimeCommitError> {
        if receipt_provider.trim().is_empty() {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "receipt_provider",
                reason: "must not be empty",
            });
        }
        if receipt_commit_id.trim().is_empty() {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "receipt_commit_id",
                reason: "must not be empty",
            });
        }

        let record = self.records_by_operation_id.get_mut(operation_id).ok_or(
            KolmeRuntimeCommitError::UnknownOperationId {
                operation_id: operation_id.to_owned(),
            },
        )?;

        if let Some(expected_provider) = record.receipt_provider.as_deref() {
            if expected_provider != receipt_provider {
                return Err(KolmeRuntimeCommitError::ReceiptFieldMismatch {
                    field: "receipt_provider",
                    expected: expected_provider.to_owned(),
                    observed: receipt_provider.to_owned(),
                });
            }
        }
        if let Some(expected_commit_id) = record.receipt_commit_id.as_deref() {
            if expected_commit_id != receipt_commit_id {
                return Err(KolmeRuntimeCommitError::ReceiptFieldMismatch {
                    field: "receipt_commit_id",
                    expected: expected_commit_id.to_owned(),
                    observed: receipt_commit_id.to_owned(),
                });
            }
        }

        let target_state = lifecycle_state_for_finality_contract(finality);

        if record.state != target_state
            && !matches!(
                (record.state, target_state),
                (
                    RuntimeCommitLifecycleState::Pending,
                    RuntimeCommitLifecycleState::Finalized
                ) | (
                    RuntimeCommitLifecycleState::Pending,
                    RuntimeCommitLifecycleState::Failed
                )
            )
        {
            return Err(KolmeRuntimeCommitError::InvalidFinalityTransition {
                from: lifecycle_state_label_contract(record.state),
                to: lifecycle_state_label_contract(target_state),
            });
        }

        record.state = target_state;
        record.needs_requeue = matches!(target_state, RuntimeCommitLifecycleState::Pending);
        record.receipt_provider = Some(receipt_provider.to_owned());
        record.receipt_commit_id = Some(receipt_commit_id.to_owned());
        if !matches!(target_state, RuntimeCommitLifecycleState::Failed) {
            record.last_error_reason = None;
        }
        Ok(record.clone())
    }

    /// Returns lifecycle record for the provided runtime operation identifier.
    pub fn record(&self, operation_id: &str) -> Option<&RuntimeCommitLifecycleRecord> {
        self.records_by_operation_id.get(operation_id)
    }

    /// Computes deterministic pending/final/failed projection counts.
    pub fn finality_projection(&self) -> RuntimeCommitFinalityProjection {
        let mut projection = RuntimeCommitFinalityProjection::default();
        for record in self.records_by_operation_id.values() {
            match record.state {
                RuntimeCommitLifecycleState::Pending => projection.pending_count += 1,
                RuntimeCommitLifecycleState::Finalized => projection.final_count += 1,
                RuntimeCommitLifecycleState::Failed => projection.failed_count += 1,
            }
        }
        projection
    }
}

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

impl KolmeRuntimeCommitNotificationEvent {
    /// Converts notification event to a provider receipt when it carries tx finality information.
    pub fn to_provider_receipt(&self, provider: &str) -> Option<KolmeRuntimeCommitProviderReceipt> {
        let provider = provider.trim();
        if provider.is_empty() {
            return None;
        }
        match self {
            Self::NewBlock {
                txhash,
                block_height,
            } => Some(KolmeRuntimeCommitProviderReceipt {
                provider: provider.to_owned(),
                commit_id: deterministic_kolme_backend_commit_id(txhash.as_str(), *block_height),
                finality: KolmeCommitReceiptFinality::Final,
            }),
            Self::FailedTransaction { txhash, .. } => Some(KolmeRuntimeCommitProviderReceipt {
                provider: provider.to_owned(),
                commit_id: deterministic_kolme_backend_commit_id(txhash.as_str(), None),
                finality: KolmeCommitReceiptFinality::Failed,
            }),
            Self::LatestBlock { .. } => None,
        }
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
        if timeout_seconds == 0 {
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
        let idempotency_key = idempotency_key.trim();
        if idempotency_key.is_empty() {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "idempotency_key must not be empty".to_owned(),
            });
        }
        let submit_path = if submit_path.trim().is_empty() {
            "/broadcast"
        } else {
            submit_path.trim()
        };
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
                map_transport_io_classification_to_provider_error(
                    classify_kolme_transport_io_error(&error),
                )
            })?;
        let timeout = Duration::from_secs(self.timeout_seconds);
        stream.set_read_timeout(Some(timeout)).map_err(|error| {
            map_transport_io_classification_to_provider_error(classify_kolme_transport_io_error(
                &error,
            ))
        })?;
        stream.set_write_timeout(Some(timeout)).map_err(|error| {
            map_transport_io_classification_to_provider_error(classify_kolme_transport_io_error(
                &error,
            ))
        })?;
        stream.write_all(request).map_err(|error| {
            map_transport_io_classification_to_provider_error(classify_kolme_transport_io_error(
                &error,
            ))
        })?;

        let mut response_bytes = Vec::new();
        stream.read_to_end(&mut response_bytes).map_err(|error| {
            map_transport_io_classification_to_provider_error(classify_kolme_transport_io_error(
                &error,
            ))
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

        if let Some(ca_file) = configured_tls_ca_file()? {
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
                map_transport_io_classification_to_provider_error(
                    classify_kolme_transport_io_error(&error),
                )
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
            map_transport_io_classification_to_provider_error(classify_kolme_transport_io_error(
                &error,
            ))
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
        if output.stdout.is_empty() {
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
        if wire_payload.trim().is_empty() {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "wire_payload must not be empty".to_owned(),
            });
        }
        if idempotency_key.trim().is_empty() {
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
        if height == 0 {
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
        if base_url.trim().is_empty() {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider_base_url",
                reason: "must not be empty",
            });
        }
        if status_path.trim().is_empty() {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider_status_path",
                reason: "must not be empty",
            });
        }
        Ok(Self {
            base_url: base_url.trim().to_owned(),
            status_path: status_path.trim().to_owned(),
            transport,
        })
    }

    /// Fetches and parses one backend finality response for the provided commit.
    pub fn check_commit_finality(
        &mut self,
        commit_id: &str,
    ) -> Result<KolmeRuntimeCommitProviderReceipt, KolmeRuntimeCommitProviderError> {
        if commit_id.trim().is_empty() {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "commit_id must not be empty".to_owned(),
            });
        }

        let response = self.transport.fetch_runtime_commit_finality(
            self.base_url.as_str(),
            self.status_path.as_str(),
            commit_id,
        )?;
        let fields = parse_kolme_provider_response_fields(response.as_str()).map_err(|error| {
            KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: error.to_string(),
            }
        })?;
        let provider = required_kolme_provider_response_field(&fields, "provider")
            .map_err(map_provider_outcome_policy_error_to_malformed)?;
        let observed_commit_id = parse_kolme_commit_id_from_response_fields(&fields)
            .map_err(map_provider_outcome_policy_error_to_malformed)?;
        if observed_commit_id != commit_id {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: format!(
                    "commit_id mismatch: expected '{commit_id}', observed '{observed_commit_id}'"
                ),
            });
        }
        let finality_value = required_kolme_provider_response_field(&fields, "finality")
            .map_err(map_provider_outcome_policy_error_to_malformed)?;
        let finality =
            parse_kolme_commit_receipt_finality(finality_value.as_str()).map_err(|error| {
                KolmeRuntimeCommitProviderError::MalformedResponse {
                    reason: error.to_string(),
                }
            })?;
        Ok(KolmeRuntimeCommitProviderReceipt {
            provider,
            commit_id: observed_commit_id,
            finality,
        })
    }

    /// Polls backend finality and returns the first non-pending receipt.
    pub fn poll_finality(
        &mut self,
        commit_id: &str,
        max_attempts: u32,
    ) -> Result<KolmeRuntimeCommitProviderReceipt, KolmeRuntimeCommitProviderError> {
        if max_attempts == 0 {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "max_attempts must be positive".to_owned(),
            });
        }
        for _ in 0..max_attempts {
            let receipt = self.check_commit_finality(commit_id)?;
            if !matches!(receipt.finality, KolmeCommitReceiptFinality::Pending) {
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
        if base_url.trim().is_empty() {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider_base_url",
                reason: "must not be empty",
            });
        }
        if provider.trim().is_empty() {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider",
                reason: "must not be empty",
            });
        }
        if max_block_lookups == 0 {
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
        Ok(Self {
            base_url: base_url.trim().to_owned(),
            block_path_template: block_path_template.trim().to_owned(),
            provider: provider.trim().to_owned(),
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
        let txhash = txhash.trim();
        if txhash.is_empty() {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "txhash must not be empty".to_owned(),
            });
        }
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
            let block = parse_kolme_block_fallback_response_contract(response.as_str())
                .map_err(|error| KolmeRuntimeCommitProviderError::MalformedResponse {
                    reason: error.to_string(),
                })
                .or_else(|_| {
                    parse_kolme_fork_block_fallback_response_contract(
                        response.as_str(),
                        self.provider.as_str(),
                        height,
                    )
                    .map_err(|error| {
                        KolmeRuntimeCommitProviderError::MalformedResponse {
                            reason: error.to_string(),
                        }
                    })
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
                .any(|value| value == txhash)
            {
                return Ok(KolmeRuntimeCommitProviderReceipt {
                    provider: self.provider.clone(),
                    commit_id: deterministic_kolme_backend_commit_id(txhash, Some(height)),
                    finality: KolmeCommitReceiptFinality::Final,
                });
            }
            if block.failed_tx_hashes.iter().any(|value| value == txhash) {
                return Ok(KolmeRuntimeCommitProviderReceipt {
                    provider: self.provider.clone(),
                    commit_id: deterministic_kolme_backend_commit_id(txhash, None),
                    finality: KolmeCommitReceiptFinality::Failed,
                });
            }
        }

        Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: format!(
                "block fallback did not resolve txhash '{txhash}' between heights {from_height} and {latest_height}"
            ),
        })
    }
}

/// Minimal websocket connector for Kolme `/notifications` consumption.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeRuntimeCommitWebsocketConnector {
    timeout_seconds: u64,
}

impl KolmeRuntimeCommitWebsocketConnector {
    /// Builds a websocket connector with deterministic timeout validation.
    pub fn new(timeout_seconds: u64) -> Result<Self, KolmeRuntimeCommitError> {
        if timeout_seconds == 0 {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "notifications_timeout_seconds",
                reason: "must be positive",
            });
        }
        Ok(Self { timeout_seconds })
    }
}

/// Websocket connection implementation used by the default notifications connector.
#[derive(Debug)]
pub struct KolmeRuntimeCommitWebsocketConnection {
    stream: TcpStream,
    read_buffer: Vec<u8>,
}

impl KolmeRuntimeCommitWebsocketConnection {
    fn new(stream: TcpStream, read_buffer: Vec<u8>) -> Self {
        Self {
            stream,
            read_buffer,
        }
    }
}

impl KolmeRuntimeCommitNotificationsConnection for KolmeRuntimeCommitWebsocketConnection {
    fn read_text_message(&mut self) -> Result<Option<String>, KolmeRuntimeCommitProviderError> {
        loop {
            if let Some(frame) = try_take_kolme_websocket_frame(&mut self.read_buffer).map_err(
                |error| match error {
                    KamnKolmeWebsocketPolicyError::Unavailable { reason } => {
                        KolmeRuntimeCommitProviderError::Unavailable { reason }
                    }
                    KamnKolmeWebsocketPolicyError::Malformed { reason } => {
                        KolmeRuntimeCommitProviderError::MalformedResponse { reason }
                    }
                },
            )? {
                match frame {
                    KamnKolmeWebsocketFrame::Text(payload_bytes) => {
                        let payload = String::from_utf8(payload_bytes).map_err(|error| {
                            KolmeRuntimeCommitProviderError::MalformedResponse {
                                reason: format!(
                                    "websocket text payload is not valid utf-8: {error}"
                                ),
                            }
                        })?;
                        return Ok(Some(payload));
                    }
                    KamnKolmeWebsocketFrame::Close => return Ok(None),
                    KamnKolmeWebsocketFrame::Ping | KamnKolmeWebsocketFrame::Pong => continue,
                }
            }

            let mut chunk = [0_u8; 1024];
            let read = self.stream.read(&mut chunk).map_err(|error| {
                map_transport_io_classification_to_provider_error(
                    classify_kolme_transport_io_error(&error),
                )
            })?;
            if read == 0 {
                return Ok(None);
            }
            self.read_buffer.extend_from_slice(&chunk[..read]);
        }
    }
}

impl KolmeRuntimeCommitNotificationsConnector for KolmeRuntimeCommitWebsocketConnector {
    type Connection = KolmeRuntimeCommitWebsocketConnection;

    fn connect(
        &mut self,
        notifications_url: &str,
    ) -> Result<Self::Connection, KolmeRuntimeCommitProviderError> {
        let endpoint = parse_kolme_websocket_endpoint(notifications_url).map_err(|error| {
            KolmeRuntimeCommitProviderError::Unavailable {
                reason: error.to_string(),
            }
        })?;
        if endpoint.secure {
            return Err(KolmeRuntimeCommitProviderError::Unavailable {
                reason: "wss:// notifications are not supported by this transport".to_owned(),
            });
        }

        let mut stream =
            TcpStream::connect((endpoint.host.as_str(), endpoint.port)).map_err(|error| {
                map_transport_io_classification_to_provider_error(
                    classify_kolme_transport_io_error(&error),
                )
            })?;
        let timeout = Duration::from_secs(self.timeout_seconds);
        stream.set_read_timeout(Some(timeout)).map_err(|error| {
            map_transport_io_classification_to_provider_error(classify_kolme_transport_io_error(
                &error,
            ))
        })?;
        stream.set_write_timeout(Some(timeout)).map_err(|error| {
            map_transport_io_classification_to_provider_error(classify_kolme_transport_io_error(
                &error,
            ))
        })?;

        let handshake = format!(
            concat!(
                "GET {} HTTP/1.1\r\n",
                "Host: {}\r\n",
                "Upgrade: websocket\r\n",
                "Connection: Upgrade\r\n",
                "Sec-WebSocket-Key: {}\r\n",
                "Sec-WebSocket-Version: 13\r\n",
                "\r\n"
            ),
            endpoint.target_path, endpoint.host_header, "dGhlIHNhbXBsZSBub25jZQ=="
        );
        stream.write_all(handshake.as_bytes()).map_err(|error| {
            map_transport_io_classification_to_provider_error(classify_kolme_transport_io_error(
                &error,
            ))
        })?;

        let mut response_bytes = Vec::new();
        let header_end = read_http_header_boundary(&mut stream, &mut response_bytes)?;
        let (header_bytes, trailing) = response_bytes.split_at(header_end + 4);
        validate_kolme_websocket_handshake_response(header_bytes).map_err(|error| match error {
            KamnKolmeWebsocketPolicyError::Unavailable { reason } => {
                KolmeRuntimeCommitProviderError::Unavailable { reason }
            }
            KamnKolmeWebsocketPolicyError::Malformed { reason } => {
                KolmeRuntimeCommitProviderError::MalformedResponse { reason }
            }
        })?;
        Ok(KolmeRuntimeCommitWebsocketConnection::new(
            stream,
            trailing.to_vec(),
        ))
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
        if provider.trim().is_empty() {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "notifications_provider",
                reason: "must not be empty",
            });
        }
        if max_reconnect_attempts == 0 {
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
            provider: provider.trim().to_owned(),
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
                            return Err(reconnect_exhausted_error(self.max_reconnect_attempts));
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
                Ok(Some(payload)) => return parse_kolme_notification_event(payload.as_str()),
                Ok(None) | Err(_) => {
                    self.connection = None;
                    reconnect_attempts += 1;
                    if reconnect_attempts >= self.max_reconnect_attempts {
                        return Err(reconnect_exhausted_error(self.max_reconnect_attempts));
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
        let expected_txhash = txhash_from_kolme_commit_id(commit_id)
            .map_err(map_provider_outcome_policy_error_to_malformed)?;

        match self.notifications_consumer.next_notification_event() {
            Ok(event) => match event {
                KolmeRuntimeCommitNotificationEvent::LatestBlock { height } => {
                    let upper_bound = if height >= from_height {
                        height.min(latest_height)
                    } else {
                        latest_height
                    };
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
                    let observed_txhash =
                        txhash_from_kolme_commit_id(receipt.commit_id.as_str())
                            .map_err(map_provider_outcome_policy_error_to_malformed)?;
                    if observed_txhash != expected_txhash {
                        return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                            reason: format!(
                                "notification txhash mismatch: expected '{expected_txhash}' observed '{observed_txhash}'"
                            ),
                        });
                    }
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
        if base_url.trim().is_empty() {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider_base_url",
                reason: "must not be empty",
            });
        }
        if submit_path.trim().is_empty() {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider_submit_path",
                reason: "must not be empty",
            });
        }
        Ok(Self {
            base_url: base_url.trim().to_owned(),
            submit_path: submit_path.trim().to_owned(),
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
        let provider_hint = provider_hint.trim();
        if provider_hint.is_empty() {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider_hint",
                reason: "must not be empty",
            });
        }
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
        parse_live_provider_response(response.as_str(), provider_hint)
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
        if expected_provider.trim().is_empty() {
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
            if receipt.provider != expected_provider {
                return Err(KolmeRuntimeCommitError::ProviderMismatch {
                    expected: expected_provider.to_owned(),
                    observed: receipt.provider,
                });
            }
            if receipt.commit_id.trim().is_empty() {
                return Err(KolmeRuntimeCommitError::InvalidRequest {
                    field: "receipt_commit_id",
                    reason: "must not be empty",
                });
            }
            if !matches!(receipt.finality, KolmeCommitReceiptFinality::Final) {
                return Err(KolmeRuntimeCommitError::NonFinalReceipt {
                    commit_id: receipt.commit_id,
                    finality: receipt.finality,
                });
            }
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

fn map_provider_outcome_policy_error_to_malformed(
    error: KamnKolmeProviderOutcomePolicyError,
) -> KolmeRuntimeCommitProviderError {
    KolmeRuntimeCommitProviderError::MalformedResponse {
        reason: error.to_string(),
    }
}

fn reconnect_exhausted_error(max_reconnect_attempts: u32) -> KolmeRuntimeCommitProviderError {
    KolmeRuntimeCommitProviderError::Unavailable {
        reason: format!(
            "notification reconnect attempts exhausted after {max_reconnect_attempts} retries"
        ),
    }
}

fn read_http_header_boundary(
    stream: &mut TcpStream,
    response_bytes: &mut Vec<u8>,
) -> Result<usize, KolmeRuntimeCommitProviderError> {
    loop {
        if let Some(position) =
            find_kolme_http_header_boundary(response_bytes).map_err(|error| match error {
                KamnKolmeWebsocketPolicyError::Unavailable { reason } => {
                    KolmeRuntimeCommitProviderError::Unavailable { reason }
                }
                KamnKolmeWebsocketPolicyError::Malformed { reason } => {
                    KolmeRuntimeCommitProviderError::MalformedResponse { reason }
                }
            })?
        {
            return Ok(position);
        }
        let mut chunk = [0_u8; 1024];
        let read = stream.read(&mut chunk).map_err(|error| {
            map_transport_io_classification_to_provider_error(classify_kolme_transport_io_error(
                &error,
            ))
        })?;
        if read == 0 {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "websocket handshake response is incomplete".to_owned(),
            });
        }
        response_bytes.extend_from_slice(&chunk[..read]);
    }
}

fn parse_kolme_notification_event(
    payload: &str,
) -> Result<KolmeRuntimeCommitNotificationEvent, KolmeRuntimeCommitProviderError> {
    let event = parse_kolme_notification_event_contract(payload).map_err(|error| {
        KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: error.to_string(),
        }
    })?;
    match event {
        KamnKolmeNotificationEvent::NewBlock {
            txhash,
            block_height,
        } => Ok(KolmeRuntimeCommitNotificationEvent::NewBlock {
            txhash,
            block_height,
        }),
        KamnKolmeNotificationEvent::FailedTransaction {
            txhash,
            proposed_height,
        } => Ok(KolmeRuntimeCommitNotificationEvent::FailedTransaction {
            txhash,
            proposed_height,
        }),
        KamnKolmeNotificationEvent::LatestBlock { height } => {
            Ok(KolmeRuntimeCommitNotificationEvent::LatestBlock { height })
        }
    }
}

fn map_transport_io_classification_to_provider_error(
    classification: KamnKolmeTransportIoClassification,
) -> KolmeRuntimeCommitProviderError {
    match classification {
        KamnKolmeTransportIoClassification::Timeout => KolmeRuntimeCommitProviderError::Timeout,
        KamnKolmeTransportIoClassification::Unavailable { reason } => {
            KolmeRuntimeCommitProviderError::Unavailable { reason }
        }
    }
}

fn configured_tls_ca_file() -> Result<Option<String>, KolmeRuntimeCommitProviderError> {
    let value = match std::env::var("KAMN_KOLME_TLS_CA_FILE") {
        Ok(value) => Some(value),
        Err(std::env::VarError::NotPresent) => return Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => {
            return Err(KolmeRuntimeCommitProviderError::Unavailable {
                reason: "KAMN_KOLME_TLS_CA_FILE must be valid utf-8".to_owned(),
            });
        }
    };
    parse_kolme_tls_ca_file_env_value(value.as_deref()).map_err(|error| match error {
        KamnKolmeTlsPolicyError::Unavailable { reason } => {
            KolmeRuntimeCommitProviderError::Unavailable { reason }
        }
    })
}

fn parse_live_provider_response(
    response: &str,
    provider_hint: Option<&str>,
) -> Result<KolmeRuntimeCommitProviderOutcome, KolmeRuntimeCommitProviderError> {
    match parse_kolme_live_provider_outcome(response, provider_hint)
        .map_err(map_provider_outcome_policy_error_to_malformed)?
    {
        KamnKolmeProviderOutcome::Submitted {
            provider,
            commit_id,
            finality,
        } => Ok(KolmeRuntimeCommitProviderOutcome::Submitted(
            KolmeRuntimeCommitProviderReceipt {
                provider,
                commit_id,
                finality: commit_finality_from_receipt_finality_contract(finality),
            },
        )),
        KamnKolmeProviderOutcome::Duplicate {
            provider,
            commit_id,
            finality,
        } => Ok(KolmeRuntimeCommitProviderOutcome::Duplicate(
            KolmeRuntimeCommitProviderReceipt {
                provider,
                commit_id,
                finality: commit_finality_from_receipt_finality_contract(finality),
            },
        )),
        KamnKolmeProviderOutcome::Rejected { reason } => {
            Ok(KolmeRuntimeCommitProviderOutcome::Rejected { reason })
        }
    }
}

/// Deterministic in-memory commit client used for contract tests and local development.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InMemoryKolmeRuntimeCommitClient {
    provider: String,
    receipts_by_idempotency_key: HashMap<String, KolmeRuntimeCommitReceipt>,
    finality_by_idempotency_key: HashMap<String, KolmeCommitReceiptFinality>,
    rejected_reasons_by_idempotency_key: HashMap<String, String>,
}

impl InMemoryKolmeRuntimeCommitClient {
    /// Constructs an in-memory commit client.
    pub fn new(provider: &str) -> Result<Self, KolmeRuntimeCommitError> {
        if provider.trim().is_empty() {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider",
                reason: "must not be empty",
            });
        }
        Ok(Self {
            provider: provider.to_owned(),
            receipts_by_idempotency_key: HashMap::new(),
            finality_by_idempotency_key: HashMap::new(),
            rejected_reasons_by_idempotency_key: HashMap::new(),
        })
    }

    /// Forces deterministic rejection for the provided idempotency key.
    pub fn reject_idempotency_key(&mut self, idempotency_key: &str, reason: &str) {
        self.rejected_reasons_by_idempotency_key
            .insert(idempotency_key.to_owned(), reason.to_owned());
    }

    /// Overrides the receipt finality that will be emitted for a given idempotency key.
    pub fn set_finality_for_idempotency_key(
        &mut self,
        idempotency_key: &str,
        finality: KolmeCommitReceiptFinality,
    ) {
        self.finality_by_idempotency_key
            .insert(idempotency_key.to_owned(), finality);
    }
}

impl KolmeRuntimeCommitClient for InMemoryKolmeRuntimeCommitClient {
    fn submit_commit(
        &mut self,
        request: &KolmeRuntimeCommitRequest,
    ) -> Result<KolmeRuntimeCommitOutcome, KolmeRuntimeCommitError> {
        request.validate()?;

        if let Some(reason) = self
            .rejected_reasons_by_idempotency_key
            .get(request.idempotency_key())
        {
            return Ok(KolmeRuntimeCommitOutcome::Rejected {
                reason: reason.clone(),
            });
        }

        if let Some(existing) = self
            .receipts_by_idempotency_key
            .get(request.idempotency_key())
        {
            return Ok(KolmeRuntimeCommitOutcome::Duplicate(existing.clone()));
        }

        let receipt = KolmeRuntimeCommitReceipt {
            provider: self.provider.clone(),
            commit_id: deterministic_runtime_commit_id_contract(
                request.operation_id.as_str(),
                request.actor_did.as_str(),
                request.nonce,
                request.payload_hash.as_str(),
            ),
            finality: self
                .finality_by_idempotency_key
                .get(request.idempotency_key())
                .copied()
                .unwrap_or(KolmeCommitReceiptFinality::Pending),
        };

        self.receipts_by_idempotency_key
            .insert(request.idempotency_key().to_owned(), receipt.clone());
        Ok(KolmeRuntimeCommitOutcome::Submitted(receipt))
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

fn lifecycle_record_from_outcome(
    request: &KolmeRuntimeCommitRequest,
    outcome: &KolmeRuntimeCommitOutcome,
) -> RuntimeCommitLifecycleRecord {
    match outcome {
        KolmeRuntimeCommitOutcome::Submitted(receipt)
        | KolmeRuntimeCommitOutcome::Duplicate(receipt) => {
            let state = lifecycle_state_for_finality_contract(receipt.finality);
            RuntimeCommitLifecycleRecord {
                operation_id: request.operation_id.clone(),
                idempotency_key: request.idempotency_key().to_owned(),
                state,
                needs_requeue: matches!(state, RuntimeCommitLifecycleState::Pending),
                receipt_provider: Some(receipt.provider.clone()),
                receipt_commit_id: Some(receipt.commit_id.clone()),
                last_error_reason: None,
            }
        }
        KolmeRuntimeCommitOutcome::Rejected { reason } => RuntimeCommitLifecycleRecord {
            operation_id: request.operation_id.clone(),
            idempotency_key: request.idempotency_key().to_owned(),
            state: RuntimeCommitLifecycleState::Failed,
            needs_requeue: false,
            receipt_provider: None,
            receipt_commit_id: None,
            last_error_reason: Some(reason.clone()),
        },
    }
}

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
