use crate::AgentDid;
use std::collections::HashMap;
use std::fmt;

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
    Rejected { reason: String },
}

/// Abstract client interface for Kolme runtime commit submission.
pub trait KolmeRuntimeCommitClient {
    /// Submits one deterministic runtime commit request.
    fn submit_commit(
        &mut self,
        request: &KolmeRuntimeCommitRequest,
    ) -> Result<KolmeRuntimeCommitOutcome, KolmeRuntimeCommitError>;
}

/// Deterministic in-memory commit client used for contract tests and local development.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InMemoryKolmeRuntimeCommitClient {
    provider: String,
    receipts_by_idempotency_key: HashMap<String, KolmeRuntimeCommitReceipt>,
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
            rejected_reasons_by_idempotency_key: HashMap::new(),
        })
    }

    /// Forces deterministic rejection for the provided idempotency key.
    pub fn reject_idempotency_key(&mut self, idempotency_key: &str, reason: &str) {
        self.rejected_reasons_by_idempotency_key
            .insert(idempotency_key.to_owned(), reason.to_owned());
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
            finality: KolmeCommitReceiptFinality::Pending,
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
}

impl fmt::Display for KolmeRuntimeCommitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest { field, reason } => {
                write!(f, "invalid runtime commit request {field}: {reason}")
            }
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
