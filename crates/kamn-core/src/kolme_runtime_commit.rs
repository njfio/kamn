//! Deterministic runtime-commit request/receipt contracts for Kolme integration.

use crate::AgentDid;
use std::collections::HashMap;
use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

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
        let idempotency_key = deterministic_idempotency_key(
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

/// Finality classification for a runtime commit receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KolmeCommitReceiptFinality {
    /// Commit has been submitted and is pending confirmation.
    Pending,
    /// Commit is fully finalized.
    Final,
    /// Commit failed validation/finality.
    Failed,
}

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeCommitLifecycleState {
    /// Commit is pending confirmation and should remain on requeue/watch.
    Pending,
    /// Commit has reached final confirmation.
    Finalized,
    /// Commit failed and should not be retried automatically.
    Failed,
}

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

        let target_state = lifecycle_state_for_finality(finality);

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
                from: lifecycle_state_label(record.state),
                to: lifecycle_state_label(target_state),
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
    transport: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedHttpEndpoint {
    host: String,
    host_header: String,
    port: u16,
    target_path: String,
}

/// Dependency-free HTTP transport implementation for runtime commit submit/finality calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeRuntimeCommitHttpTransport {
    timeout_seconds: u64,
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
        Ok(Self { timeout_seconds })
    }

    fn execute_request(
        &self,
        base_url: &str,
        path: &str,
        method: &str,
        body: Option<&str>,
        headers: &[(&str, &str)],
    ) -> Result<String, KolmeRuntimeCommitProviderError> {
        let endpoint = parse_http_endpoint(base_url, path)?;
        let mut stream = TcpStream::connect((endpoint.host.as_str(), endpoint.port))
            .map_err(map_transport_io_error)?;

        let timeout = Duration::from_secs(self.timeout_seconds);
        stream
            .set_read_timeout(Some(timeout))
            .map_err(map_transport_io_error)?;
        stream
            .set_write_timeout(Some(timeout))
            .map_err(map_transport_io_error)?;

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
        if body.is_some() {
            request.push_str(format!("Content-Length: {}\r\n", payload.len()).as_str());
        }
        request.push_str("\r\n");
        if body.is_some() {
            request.push_str(payload);
        }

        stream
            .write_all(request.as_bytes())
            .map_err(map_transport_io_error)?;

        let mut response_bytes = Vec::new();
        stream
            .read_to_end(&mut response_bytes)
            .map_err(map_transport_io_error)?;
        parse_http_response(response_bytes)
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
        if commit_id.trim().is_empty() {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "commit_id must not be empty".to_owned(),
            });
        }
        let encoded_commit_id = percent_encode(commit_id);
        let separator = if status_path.contains('?') { "&" } else { "?" };
        let path = format!("{status_path}{separator}commit_id={encoded_commit_id}");
        self.execute_request(base_url, path.as_str(), "GET", None, &[])
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
        let fields = parse_response_fields(response.as_str())?;
        let provider = required_response_field(&fields, "provider")?;
        let observed_commit_id = resolve_commit_id(&fields)?;
        if observed_commit_id != commit_id {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: format!(
                    "commit_id mismatch: expected '{commit_id}', observed '{observed_commit_id}'"
                ),
            });
        }
        let finality_value = required_response_field(&fields, "finality")?;
        let finality = parse_receipt_finality(finality_value.as_str())?;
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
            transport,
        })
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
        parse_live_provider_response(response.as_str())
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
        let provider_outcome = self
            .provider
            .submit_runtime_commit(&request.to_wire_payload(), request.idempotency_key())
            .map_err(map_provider_error)?;
        map_provider_outcome(provider_outcome, self.expected_provider.as_str())
    }
}

fn map_provider_error(error: KolmeRuntimeCommitProviderError) -> KolmeRuntimeCommitError {
    match error {
        KolmeRuntimeCommitProviderError::Timeout => KolmeRuntimeCommitError::ProviderTransport {
            kind: KolmeRuntimeCommitTransportErrorKind::Timeout,
            detail: "provider request timed out".to_owned(),
        },
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
    }
}

fn parse_http_endpoint(
    base_url: &str,
    path: &str,
) -> Result<ParsedHttpEndpoint, KolmeRuntimeCommitProviderError> {
    let base = base_url.trim();
    if base.is_empty() {
        return Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "base_url must not be empty".to_owned(),
        });
    }
    if !base.starts_with("http://") {
        return Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "only http:// base_url is supported".to_owned(),
        });
    }

    let remainder = &base["http://".len()..];
    let (authority, base_path) = match remainder.split_once('/') {
        Some((left, right)) => (left, format!("/{}", right.trim_start_matches('/'))),
        None => (remainder, "/".to_owned()),
    };

    if authority.is_empty() {
        return Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "base_url host must not be empty".to_owned(),
        });
    }

    let (host, port) = parse_authority(authority)?;
    let target_path = join_http_paths(base_path.as_str(), path);
    Ok(ParsedHttpEndpoint {
        host,
        host_header: authority.to_owned(),
        port,
        target_path,
    })
}

fn parse_authority(authority: &str) -> Result<(String, u16), KolmeRuntimeCommitProviderError> {
    if authority.starts_with('[') {
        return Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "ipv6 host syntax is not supported".to_owned(),
        });
    }
    if let Some((host, port_raw)) = authority.rsplit_once(':') {
        if !port_raw.is_empty() && port_raw.chars().all(|ch| ch.is_ascii_digit()) {
            let port = port_raw.parse::<u16>().map_err(|_| {
                KolmeRuntimeCommitProviderError::Unavailable {
                    reason: "base_url port is invalid".to_owned(),
                }
            })?;
            if host.trim().is_empty() {
                return Err(KolmeRuntimeCommitProviderError::Unavailable {
                    reason: "base_url host must not be empty".to_owned(),
                });
            }
            return Ok((host.to_owned(), port));
        }
        if !port_raw.is_empty() {
            return Err(KolmeRuntimeCommitProviderError::Unavailable {
                reason: "base_url port is invalid".to_owned(),
            });
        }
    }
    Ok((authority.to_owned(), 80))
}

fn join_http_paths(base_path: &str, request_path: &str) -> String {
    let base = if base_path.trim().is_empty() {
        "/".to_owned()
    } else if base_path.starts_with('/') {
        base_path.to_owned()
    } else {
        format!("/{base_path}")
    };

    let request = request_path.trim();
    if request.is_empty() || request == "/" {
        return base;
    }

    if request.starts_with('/') {
        if base == "/" {
            return request.to_owned();
        }
        return format!("{}{}", base.trim_end_matches('/'), request);
    }

    if base == "/" {
        format!("/{request}")
    } else {
        format!("{}/{}", base.trim_end_matches('/'), request)
    }
}

fn map_transport_io_error(error: std::io::Error) -> KolmeRuntimeCommitProviderError {
    if matches!(
        error.kind(),
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
    ) {
        return KolmeRuntimeCommitProviderError::Timeout;
    }
    KolmeRuntimeCommitProviderError::Unavailable {
        reason: format!("transport io error: {error}"),
    }
}

fn parse_http_response(response_bytes: Vec<u8>) -> Result<String, KolmeRuntimeCommitProviderError> {
    let response_text = String::from_utf8(response_bytes).map_err(|error| {
        KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: format!("http response body is not valid utf-8: {error}"),
        }
    })?;

    let (raw_headers, raw_body) = response_text.split_once("\r\n\r\n").ok_or_else(|| {
        KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "http response missing header/body separator".to_owned(),
        }
    })?;

    let mut header_lines = raw_headers.lines();
    let status_line =
        header_lines
            .next()
            .ok_or_else(|| KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "http response missing status line".to_owned(),
            })?;
    let mut status_parts = status_line.split_whitespace();
    let _http_version =
        status_parts
            .next()
            .ok_or_else(|| KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "http response status line is malformed".to_owned(),
            })?;
    let status_code_raw =
        status_parts
            .next()
            .ok_or_else(|| KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "http response status code is missing".to_owned(),
            })?;
    let status_code = status_code_raw.parse::<u16>().map_err(|_| {
        KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: format!("http response status code is invalid: {status_code_raw}"),
        }
    })?;

    if status_code == 408 || status_code == 504 {
        return Err(KolmeRuntimeCommitProviderError::Timeout);
    }
    if status_code >= 500 {
        return Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: format!("http response status indicates upstream failure: {status_code}"),
        });
    }

    let mut declared_content_length = None;
    for line in header_lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.eq_ignore_ascii_case("Content-Length") {
            let parsed = value.trim().parse::<usize>().map_err(|_| {
                KolmeRuntimeCommitProviderError::MalformedResponse {
                    reason: "http response content-length is invalid".to_owned(),
                }
            })?;
            declared_content_length = Some(parsed);
            break;
        }
    }
    if let Some(declared) = declared_content_length {
        let observed = raw_body.len();
        if declared != observed {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: format!(
                    "http response content-length mismatch: declared {declared}, observed {observed}"
                ),
            });
        }
    }

    Ok(raw_body.to_owned())
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        let ch = byte as char;
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '~') {
            encoded.push(ch);
        } else {
            encoded.push('%');
            encoded.push_str(format!("{byte:02X}").as_str());
        }
    }
    encoded
}

fn parse_live_provider_response(
    response: &str,
) -> Result<KolmeRuntimeCommitProviderOutcome, KolmeRuntimeCommitProviderError> {
    let fields = parse_response_fields(response)?;
    let status = required_response_field(&fields, "status")?;
    match status.as_str() {
        "submitted" | "duplicate" => {
            let provider = required_response_field(&fields, "provider")?;
            let commit_id = resolve_commit_id(&fields)?;
            let finality_value = required_response_field(&fields, "finality")?;
            let finality = parse_receipt_finality(finality_value.as_str())?;
            let receipt = KolmeRuntimeCommitProviderReceipt {
                provider,
                commit_id,
                finality,
            };
            if status == "submitted" {
                Ok(KolmeRuntimeCommitProviderOutcome::Submitted(receipt))
            } else {
                Ok(KolmeRuntimeCommitProviderOutcome::Duplicate(receipt))
            }
        }
        "rejected" => {
            let reason = required_response_field(&fields, "reason")?;
            Ok(KolmeRuntimeCommitProviderOutcome::Rejected { reason })
        }
        _ => Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: format!("invalid status value: {status}"),
        }),
    }
}

fn parse_response_fields(
    response: &str,
) -> Result<HashMap<String, String>, KolmeRuntimeCommitProviderError> {
    let trimmed = response.trim();
    if trimmed.is_empty() {
        return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "response body must not be empty".to_owned(),
        });
    }

    if trimmed.starts_with('{') {
        return parse_flat_json_response_fields(trimmed);
    }

    parse_key_value_response_fields(trimmed)
}

fn parse_key_value_response_fields(
    response: &str,
) -> Result<HashMap<String, String>, KolmeRuntimeCommitProviderError> {
    let mut fields = HashMap::new();
    for line in response.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (key, value) = trimmed.split_once('=').ok_or_else(|| {
            KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: format!("invalid key/value response line: {trimmed}"),
            }
        })?;
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: format!("invalid key/value response line: {trimmed}"),
            });
        }
        fields.insert(key.to_owned(), value.to_owned());
    }
    if fields.is_empty() {
        return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "response body must contain at least one field".to_owned(),
        });
    }
    Ok(fields)
}

fn parse_flat_json_response_fields(
    response: &str,
) -> Result<HashMap<String, String>, KolmeRuntimeCommitProviderError> {
    let body = response.trim();
    if !(body.starts_with('{') && body.ends_with('}')) {
        return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "json response must be an object".to_owned(),
        });
    }
    let inner = &body[1..body.len() - 1];
    if inner.trim().is_empty() {
        return Ok(HashMap::new());
    }

    let entries = split_unquoted(inner, ',').map_err(|reason| {
        KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: format!("invalid json response: {reason}"),
        }
    })?;

    let mut fields = HashMap::new();
    for entry in entries {
        let pair = split_unquoted(entry.as_str(), ':').map_err(|reason| {
            KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: format!("invalid json response pair: {reason}"),
            }
        })?;
        if pair.len() != 2 {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "json response pair must contain exactly one ':'".to_owned(),
            });
        }

        let key = parse_json_string(pair[0].trim()).map_err(|reason| {
            KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: format!("invalid json key: {reason}"),
            }
        })?;
        let value = parse_json_string(pair[1].trim()).map_err(|reason| {
            KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: format!("invalid json value: {reason}"),
            }
        })?;
        fields.insert(key, value);
    }
    Ok(fields)
}

fn split_unquoted(input: &str, delimiter: char) -> Result<Vec<String>, &'static str> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escape = false;

    for ch in input.chars() {
        if escape {
            current.push(ch);
            escape = false;
            continue;
        }

        if ch == '\\' && in_quotes {
            current.push(ch);
            escape = true;
            continue;
        }

        if ch == '"' {
            in_quotes = !in_quotes;
            current.push(ch);
            continue;
        }

        if ch == delimiter && !in_quotes {
            if current.trim().is_empty() {
                return Err("empty segment");
            }
            parts.push(current.trim().to_owned());
            current.clear();
            continue;
        }

        current.push(ch);
    }

    if in_quotes {
        return Err("unterminated quoted string");
    }
    if current.trim().is_empty() {
        return Err("empty trailing segment");
    }
    parts.push(current.trim().to_owned());
    Ok(parts)
}

fn parse_json_string(token: &str) -> Result<String, &'static str> {
    let trimmed = token.trim();
    if !(trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2) {
        return Err("token must be a quoted string");
    }
    let mut output = String::new();
    let mut escape = false;
    for ch in trimmed[1..trimmed.len() - 1].chars() {
        if escape {
            let mapped = match ch {
                '\\' => '\\',
                '"' => '"',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                _ => return Err("unsupported escape sequence"),
            };
            output.push(mapped);
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        output.push(ch);
    }
    if escape {
        return Err("unterminated escape sequence");
    }
    Ok(output)
}

fn required_response_field(
    fields: &HashMap<String, String>,
    field: &'static str,
) -> Result<String, KolmeRuntimeCommitProviderError> {
    let value =
        fields
            .get(field)
            .ok_or_else(|| KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: format!("missing required field: {field}"),
            })?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: format!("field must not be empty: {field}"),
        });
    }
    Ok(trimmed.to_owned())
}

fn resolve_commit_id(
    fields: &HashMap<String, String>,
) -> Result<String, KolmeRuntimeCommitProviderError> {
    if let Some(commit_id) = fields.get("commit_id") {
        let trimmed = commit_id.trim();
        if trimmed.is_empty() {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "field must not be empty: commit_id".to_owned(),
            });
        }
        return Ok(trimmed.to_owned());
    }

    let tx_hash = fields
        .get("tx_hash")
        .or_else(|| fields.get("txhash"))
        .ok_or_else(|| KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "missing required field: commit_id".to_owned(),
        })?;
    let tx_hash = tx_hash.trim();
    if tx_hash.is_empty() {
        return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "field must not be empty: tx_hash".to_owned(),
        });
    }

    let block_height = match fields.get("block_height") {
        Some(raw_height) => Some(parse_block_height(raw_height.as_str())?),
        None => None,
    };
    Ok(deterministic_backend_commit_id(tx_hash, block_height))
}

fn parse_block_height(raw: &str) -> Result<u64, KolmeRuntimeCommitProviderError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "field must not be empty: block_height".to_owned(),
        });
    }
    let height =
        trimmed
            .parse::<u64>()
            .map_err(|_| KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: format!("invalid block_height value: {trimmed}"),
            })?;
    if height == 0 {
        return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "block_height must be positive".to_owned(),
        });
    }
    Ok(height)
}

fn deterministic_backend_commit_id(tx_hash: &str, block_height: Option<u64>) -> String {
    match block_height {
        Some(height) => format!("kolme-commit:{tx_hash}:h{height}"),
        None => format!("kolme-commit:{tx_hash}"),
    }
}

fn parse_receipt_finality(
    value: &str,
) -> Result<KolmeCommitReceiptFinality, KolmeRuntimeCommitProviderError> {
    match value {
        "pending" | "accepted" | "mempool" => Ok(KolmeCommitReceiptFinality::Pending),
        "final" | "confirmed" | "finalized" => Ok(KolmeCommitReceiptFinality::Final),
        "failed" | "rejected" => Ok(KolmeCommitReceiptFinality::Failed),
        _ => Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: format!("invalid finality value: {value}"),
        }),
    }
}

fn map_provider_outcome(
    outcome: KolmeRuntimeCommitProviderOutcome,
    expected_provider: &str,
) -> Result<KolmeRuntimeCommitOutcome, KolmeRuntimeCommitError> {
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => {
            let runtime_receipt = map_provider_receipt(receipt, expected_provider)?;
            Ok(KolmeRuntimeCommitOutcome::Submitted(runtime_receipt))
        }
        KolmeRuntimeCommitProviderOutcome::Duplicate(receipt) => {
            let runtime_receipt = map_provider_receipt(receipt, expected_provider)?;
            Ok(KolmeRuntimeCommitOutcome::Duplicate(runtime_receipt))
        }
        KolmeRuntimeCommitProviderOutcome::Rejected { reason } => {
            Ok(KolmeRuntimeCommitOutcome::Rejected { reason })
        }
    }
}

fn map_provider_receipt(
    receipt: KolmeRuntimeCommitProviderReceipt,
    expected_provider: &str,
) -> Result<KolmeRuntimeCommitReceipt, KolmeRuntimeCommitError> {
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
            commit_id: deterministic_commit_id(
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
                commit_finality_label(*finality)
            ),
        }
    }
}

impl std::error::Error for KolmeRuntimeCommitError {}

fn deterministic_idempotency_key(
    operation_id: &str,
    state_root: &str,
    actor_did: &str,
    nonce: u64,
    payload_hash: &str,
) -> String {
    format!(
        "kolme-runtime-commit:{}:{}:{}:{}:{}",
        operation_id.trim(),
        state_root.trim(),
        actor_did.trim(),
        nonce,
        payload_hash.trim().len()
    )
}

fn deterministic_commit_id(
    operation_id: &str,
    actor_did: &str,
    nonce: u64,
    payload_hash: &str,
) -> String {
    format!(
        "kolme-commit:{}:{}:{}:{}",
        operation_id,
        actor_did,
        nonce,
        payload_hash.len()
    )
}

fn lifecycle_state_for_finality(
    finality: KolmeCommitReceiptFinality,
) -> RuntimeCommitLifecycleState {
    match finality {
        KolmeCommitReceiptFinality::Pending => RuntimeCommitLifecycleState::Pending,
        KolmeCommitReceiptFinality::Final => RuntimeCommitLifecycleState::Finalized,
        KolmeCommitReceiptFinality::Failed => RuntimeCommitLifecycleState::Failed,
    }
}

fn lifecycle_state_label(state: RuntimeCommitLifecycleState) -> &'static str {
    match state {
        RuntimeCommitLifecycleState::Pending => "pending",
        RuntimeCommitLifecycleState::Finalized => "finalized",
        RuntimeCommitLifecycleState::Failed => "failed",
    }
}

fn commit_finality_label(finality: KolmeCommitReceiptFinality) -> &'static str {
    match finality {
        KolmeCommitReceiptFinality::Pending => "pending",
        KolmeCommitReceiptFinality::Final => "final",
        KolmeCommitReceiptFinality::Failed => "failed",
    }
}

fn lifecycle_record_from_outcome(
    request: &KolmeRuntimeCommitRequest,
    outcome: &KolmeRuntimeCommitOutcome,
) -> RuntimeCommitLifecycleRecord {
    match outcome {
        KolmeRuntimeCommitOutcome::Submitted(receipt)
        | KolmeRuntimeCommitOutcome::Duplicate(receipt) => {
            let state = lifecycle_state_for_finality(receipt.finality);
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
    use super::{KolmeRuntimeCommitError, KolmeRuntimeCommitRequest};

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
}
