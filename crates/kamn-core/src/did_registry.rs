//! DID registry lifecycle, idempotent submission, and finality tracking contracts.

use crate::{AgentDid, DidDocument};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
struct DidRegistryRecord {
    document: DidDocument,
    revoked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Lifecycle mutation action applied to a DID record.
pub enum DidLifecycleMutationAction {
    /// Rotate to a new active DID document.
    Rotate {
        /// Replacement DID document.
        document: DidDocument,
    },
    /// Revoke the DID record.
    Revoke,
    /// Recover a revoked DID with replacement document.
    Recover {
        /// Recovery DID document.
        document: DidDocument,
    },
}

impl DidLifecycleMutationAction {
    fn label(&self) -> &'static str {
        match self {
            Self::Rotate { .. } => "rotate",
            Self::Revoke => "revoke",
            Self::Recover { .. } => "recover",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Lifecycle mutation request envelope.
pub struct DidLifecycleMutationRequest {
    /// Target DID for mutation.
    pub did: AgentDid,
    /// Actor DID authorized to perform mutation.
    pub actor_did: String,
    /// Strictly increasing mutation nonce.
    pub nonce: u64,
    /// Requested lifecycle action.
    pub action: DidLifecycleMutationAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Evidence produced by successful lifecycle mutation.
pub struct DidLifecycleMutationEvidence {
    /// Target DID identifier.
    pub did: String,
    /// Actor DID that executed mutation.
    pub actor_did: String,
    /// Mutation nonce accepted by registry.
    pub nonce: u64,
    /// Action label executed by registry.
    pub action: &'static str,
    /// Revocation state before mutation.
    pub from_revoked: bool,
    /// Revocation state after mutation.
    pub to_revoked: bool,
    /// Stable reason code for policy lanes.
    pub reason_code: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Retry classification for register submissions.
pub enum DidSubmissionRetryClass {
    /// First submission for DID/key pair.
    NewSubmission,
    /// Submission in-flight and retry is allowed.
    RetryableInFlight,
    /// Submission already finalized and should not retry.
    FinalizedNoRetry,
    /// Idempotency key conflicts with existing submission.
    ConflictNoRetry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Finality status for DID registration submission.
pub enum DidSubmissionFinalityStatus {
    /// Submission finalized successfully.
    Confirmed,
    /// Submission finalized as rejected.
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Finality record tracked per DID submission.
pub struct DidSubmissionFinalityRecord {
    /// Idempotency key associated with submission.
    pub idempotency_key: String,
    /// Monotonic finality sequence number.
    pub sequence: u64,
    /// Finality status.
    pub status: DidSubmissionFinalityStatus,
    /// Provider receipt payload for finality event.
    pub receipt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Chain adapter request for DID registration submission.
pub struct DidChainSubmissionRequest {
    /// DID being submitted.
    pub did: AgentDid,
    /// Deterministic idempotency key.
    pub idempotency_key: String,
    /// DID document payload for registration.
    pub document: DidDocument,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Chain adapter receipt for a submission attempt.
pub struct DidChainSubmissionReceipt {
    /// Provider name that handled submission.
    pub provider: String,
    /// Provider transaction identifier.
    pub transaction_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Submission outcome returned by chain adapter.
pub enum DidChainSubmissionOutcome {
    /// New submission accepted by provider.
    Submitted(DidChainSubmissionReceipt),
    /// Duplicate idempotency key acknowledged with existing receipt.
    Duplicate(DidChainSubmissionReceipt),
    /// Submission rejected by provider policy.
    Rejected {
        /// Provider-supplied rejection reason.
        reason: String,
    },
    /// Registry determined no provider call was needed.
    FinalizedNoOp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Result envelope for registry + chain adapter registration flow.
pub struct DidChainSubmissionResult {
    /// DID processed by submission flow.
    pub did: AgentDid,
    /// Idempotency key used for this flow.
    pub idempotency_key: String,
    /// Retry classification returned by registry.
    pub retry_class: DidSubmissionRetryClass,
    /// Provider/registry submission outcome.
    pub outcome: DidChainSubmissionOutcome,
}

/// Chain adapter abstraction for DID registration backends.
pub trait DidRegistrationChainAdapter {
    /// Submits a DID registration request via backing provider.
    fn submit_registration(
        &mut self,
        request: &DidChainSubmissionRequest,
    ) -> Result<DidChainSubmissionOutcome, DidRegistryError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// In-memory chain adapter used for deterministic tests.
pub struct InMemoryDidRegistrationChainAdapter {
    provider: String,
    receipts_by_key: HashMap<String, DidChainSubmissionReceipt>,
    rejected_reasons_by_key: HashMap<String, String>,
}

impl InMemoryDidRegistrationChainAdapter {
    /// Creates an in-memory adapter with provider label.
    pub fn new(provider: &str) -> Self {
        Self {
            provider: provider.to_owned(),
            receipts_by_key: HashMap::new(),
            rejected_reasons_by_key: HashMap::new(),
        }
    }

    /// Configures an idempotency key to return a rejected outcome.
    pub fn reject_idempotency_key(&mut self, idempotency_key: &str, reason: &str) {
        self.rejected_reasons_by_key
            .insert(idempotency_key.to_owned(), reason.to_owned());
    }
}

impl DidRegistrationChainAdapter for InMemoryDidRegistrationChainAdapter {
    fn submit_registration(
        &mut self,
        request: &DidChainSubmissionRequest,
    ) -> Result<DidChainSubmissionOutcome, DidRegistryError> {
        if let Some(reason) = self.rejected_reasons_by_key.get(&request.idempotency_key) {
            return Ok(DidChainSubmissionOutcome::Rejected {
                reason: reason.clone(),
            });
        }

        if let Some(existing) = self.receipts_by_key.get(&request.idempotency_key) {
            return Ok(DidChainSubmissionOutcome::Duplicate(existing.clone()));
        }

        let receipt = DidChainSubmissionReceipt {
            provider: self.provider.clone(),
            transaction_id: format!(
                "did-tx:{}:{}",
                request.did.method_specific_id(),
                request.idempotency_key.len()
            ),
        };
        self.receipts_by_key
            .insert(request.idempotency_key.clone(), receipt.clone());
        Ok(DidChainSubmissionOutcome::Submitted(receipt))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// DID registry state machine and submission/finality index.
pub struct DidRegistry {
    records: HashMap<AgentDid, DidRegistryRecord>,
    submission_keys_by_did: HashMap<AgentDid, String>,
    finality_by_did: HashMap<AgentDid, DidSubmissionFinalityRecord>,
    last_mutation_nonce_by_did: HashMap<AgentDid, u64>,
}

impl DidRegistry {
    /// Creates an empty DID registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new DID document in active state.
    pub fn register(
        &mut self,
        did: AgentDid,
        document: DidDocument,
    ) -> Result<(), DidRegistryError> {
        Self::validate_document_did(&did, &document)?;
        let did_id = did.as_str().to_owned();
        match self.records.get(&did) {
            Some(record) if !record.revoked => Err(DidRegistryError::AlreadyRegistered(did_id)),
            Some(_) => Err(DidRegistryError::Revoked(did_id)),
            None => {
                self.records.insert(
                    did,
                    DidRegistryRecord {
                        document,
                        revoked: false,
                    },
                );
                Ok(())
            }
        }
    }

    /// Resolves active DID document by identifier.
    pub fn resolve(&self, did: &AgentDid) -> Result<&DidDocument, DidRegistryError> {
        match self.records.get(did) {
            Some(record) if !record.revoked => Ok(&record.document),
            Some(_) => Err(DidRegistryError::Revoked(did.as_str().to_owned())),
            None => Err(DidRegistryError::NotFound(did.as_str().to_owned())),
        }
    }

    /// Replaces document for an active DID.
    pub fn update(&mut self, did: AgentDid, document: DidDocument) -> Result<(), DidRegistryError> {
        Self::validate_document_did(&did, &document)?;
        match self.records.get_mut(&did) {
            Some(record) if record.revoked => {
                Err(DidRegistryError::Revoked(did.as_str().to_owned()))
            }
            Some(record) => {
                record.document = document;
                Ok(())
            }
            None => Err(DidRegistryError::NotFound(did.as_str().to_owned())),
        }
    }

    /// Revokes an active DID.
    pub fn revoke(&mut self, did: &AgentDid) -> Result<(), DidRegistryError> {
        match self.records.get_mut(did) {
            Some(record) if record.revoked => {
                Err(DidRegistryError::Revoked(did.as_str().to_owned()))
            }
            Some(record) => {
                record.revoked = true;
                Ok(())
            }
            None => Err(DidRegistryError::NotFound(did.as_str().to_owned())),
        }
    }

    /// Computes deterministic idempotency key for register request.
    pub fn idempotency_key_for_register(
        &self,
        did: &AgentDid,
        document: &DidDocument,
    ) -> Result<String, DidRegistryError> {
        Self::validate_document_did(did, document)?;

        let capability_fingerprint = document.metadata.capabilities.join(",");
        let verification_fingerprint = document
            .verification_method
            .iter()
            .map(|verification| {
                format!(
                    "{}:{}:{}",
                    verification.id, verification.type_name, verification.public_key_multibase
                )
            })
            .collect::<Vec<_>>()
            .join("|");
        let service_fingerprint = document
            .service
            .iter()
            .map(|service| {
                format!(
                    "{}:{}:{}",
                    service.id, service.type_name, service.service_endpoint
                )
            })
            .collect::<Vec<_>>()
            .join("|");

        Ok(format!(
            "did-register:{}:{}:{}:{}:{}:{}",
            did.as_str(),
            document.metadata.agent_type,
            document.metadata.model_family,
            capability_fingerprint,
            verification_fingerprint,
            service_fingerprint
        ))
    }

    /// Classifies retry posture for register operation.
    pub fn classify_register_retry(
        &self,
        did: &AgentDid,
        document: &DidDocument,
    ) -> Result<DidSubmissionRetryClass, DidRegistryError> {
        let idempotency_key = self.idempotency_key_for_register(did, document)?;
        Ok(self.classify_retry_by_key(did, &idempotency_key))
    }

    /// Registers DID with built-in retry classification guard.
    pub fn register_with_retry_guard(
        &mut self,
        did: AgentDid,
        document: DidDocument,
    ) -> Result<DidSubmissionRetryClass, DidRegistryError> {
        let idempotency_key = self.idempotency_key_for_register(&did, &document)?;
        match self.classify_retry_by_key(&did, &idempotency_key) {
            DidSubmissionRetryClass::NewSubmission => {
                self.register(did.clone(), document)?;
                self.submission_keys_by_did.insert(did, idempotency_key);
                Ok(DidSubmissionRetryClass::NewSubmission)
            }
            DidSubmissionRetryClass::RetryableInFlight => {
                Ok(DidSubmissionRetryClass::RetryableInFlight)
            }
            DidSubmissionRetryClass::FinalizedNoRetry => {
                Ok(DidSubmissionRetryClass::FinalizedNoRetry)
            }
            DidSubmissionRetryClass::ConflictNoRetry => {
                let existing_key = self
                    .submission_keys_by_did
                    .get(&did)
                    .cloned()
                    .unwrap_or_default();
                Err(DidRegistryError::ConflictingSubmissionIdempotencyKey {
                    did: did.as_str().to_owned(),
                    existing_key,
                    provided_key: idempotency_key,
                })
            }
        }
    }

    /// Executes register flow through chain adapter with retry guard.
    pub fn submit_registration_via_chain_adapter<A: DidRegistrationChainAdapter>(
        &mut self,
        adapter: &mut A,
        did: AgentDid,
        document: DidDocument,
    ) -> Result<DidChainSubmissionResult, DidRegistryError> {
        let idempotency_key = self.idempotency_key_for_register(&did, &document)?;
        let retry_class = self.register_with_retry_guard(did.clone(), document.clone())?;

        let outcome = if retry_class == DidSubmissionRetryClass::FinalizedNoRetry {
            DidChainSubmissionOutcome::FinalizedNoOp
        } else {
            let request = DidChainSubmissionRequest {
                did: did.clone(),
                idempotency_key: idempotency_key.clone(),
                document,
            };
            adapter.submit_registration(&request)?
        };

        Ok(DidChainSubmissionResult {
            did,
            idempotency_key,
            retry_class,
            outcome,
        })
    }

    /// Records finality update for prior register submission.
    pub fn record_register_finality(
        &mut self,
        did: &AgentDid,
        idempotency_key: &str,
        sequence: u64,
        status: DidSubmissionFinalityStatus,
        receipt: &str,
    ) -> Result<(), DidRegistryError> {
        let Some(expected_key) = self.submission_keys_by_did.get(did) else {
            return Err(DidRegistryError::UnknownSubmissionIdempotencyKey {
                did: did.as_str().to_owned(),
                idempotency_key: idempotency_key.to_owned(),
            });
        };
        if expected_key != idempotency_key {
            return Err(DidRegistryError::UnknownSubmissionIdempotencyKey {
                did: did.as_str().to_owned(),
                idempotency_key: idempotency_key.to_owned(),
            });
        }

        if let Some(current) = self.finality_by_did.get(did) {
            if sequence < current.sequence {
                return Err(DidRegistryError::StaleFinalityUpdate {
                    did: did.as_str().to_owned(),
                    current_sequence: current.sequence,
                    attempted_sequence: sequence,
                });
            }

            if sequence == current.sequence {
                if current.idempotency_key == idempotency_key
                    && current.status == status
                    && current.receipt == receipt
                {
                    return Ok(());
                }

                return Err(DidRegistryError::ConflictingFinalityUpdate {
                    did: did.as_str().to_owned(),
                    sequence,
                });
            }

            if current.idempotency_key != idempotency_key {
                return Err(DidRegistryError::ConflictingFinalityUpdate {
                    did: did.as_str().to_owned(),
                    sequence,
                });
            }
        }

        self.finality_by_did.insert(
            did.clone(),
            DidSubmissionFinalityRecord {
                idempotency_key: idempotency_key.to_owned(),
                sequence,
                status,
                receipt: receipt.to_owned(),
            },
        );
        Ok(())
    }

    /// Returns most recent finality record for DID, if present.
    pub fn register_finality(&self, did: &AgentDid) -> Option<&DidSubmissionFinalityRecord> {
        self.finality_by_did.get(did)
    }

    /// Applies lifecycle mutation with nonce and actor authorization checks.
    pub fn apply_lifecycle_mutation(
        &mut self,
        request: DidLifecycleMutationRequest,
    ) -> Result<DidLifecycleMutationEvidence, DidRegistryError> {
        let did = request.did;
        let did_id = did.as_str().to_owned();

        if request.nonce == 0 {
            return Err(DidRegistryError::InvalidMutationNonce {
                did: did_id,
                nonce: request.nonce,
            });
        }

        if let Some(last_nonce) = self.last_mutation_nonce_by_did.get(&did) {
            if request.nonce <= *last_nonce {
                return Err(DidRegistryError::ReplayedMutationNonce {
                    did: did.as_str().to_owned(),
                    last_nonce: *last_nonce,
                    found: request.nonce,
                });
            }
        }

        let from_revoked = self
            .records
            .get(&did)
            .map(|record| record.revoked)
            .ok_or_else(|| DidRegistryError::NotFound(did.as_str().to_owned()))?;

        self.authorize_mutation_actor(&did, &request.actor_did)?;
        let action = request.action.label();

        match request.action {
            DidLifecycleMutationAction::Rotate { document } => {
                if from_revoked {
                    return Err(DidRegistryError::InvalidLifecycleMutationTransition {
                        did: did.as_str().to_owned(),
                        action,
                        from_revoked,
                    });
                }

                Self::validate_document_did(&did, &document)?;
                let record = self
                    .records
                    .get_mut(&did)
                    .ok_or_else(|| DidRegistryError::NotFound(did.as_str().to_owned()))?;
                record.document = document;
            }
            DidLifecycleMutationAction::Revoke => {
                if from_revoked {
                    return Err(DidRegistryError::InvalidLifecycleMutationTransition {
                        did: did.as_str().to_owned(),
                        action,
                        from_revoked,
                    });
                }

                let record = self
                    .records
                    .get_mut(&did)
                    .ok_or_else(|| DidRegistryError::NotFound(did.as_str().to_owned()))?;
                record.revoked = true;
            }
            DidLifecycleMutationAction::Recover { document } => {
                if !from_revoked {
                    return Err(DidRegistryError::InvalidLifecycleMutationTransition {
                        did: did.as_str().to_owned(),
                        action,
                        from_revoked,
                    });
                }

                Self::validate_document_did(&did, &document)?;
                let record = self
                    .records
                    .get_mut(&did)
                    .ok_or_else(|| DidRegistryError::NotFound(did.as_str().to_owned()))?;
                record.document = document;
                record.revoked = false;
            }
        }

        self.last_mutation_nonce_by_did
            .insert(did.clone(), request.nonce);
        let to_revoked = self
            .records
            .get(&did)
            .map(|record| record.revoked)
            .unwrap_or(false);

        Ok(DidLifecycleMutationEvidence {
            did: did.as_str().to_owned(),
            actor_did: request.actor_did,
            nonce: request.nonce,
            action,
            from_revoked,
            to_revoked,
            reason_code: "did_lifecycle_mutation_allowed",
        })
    }

    fn classify_retry_by_key(
        &self,
        did: &AgentDid,
        idempotency_key: &str,
    ) -> DidSubmissionRetryClass {
        let Some(existing_key) = self.submission_keys_by_did.get(did) else {
            return DidSubmissionRetryClass::NewSubmission;
        };

        if existing_key != idempotency_key {
            return DidSubmissionRetryClass::ConflictNoRetry;
        }

        if self.finality_by_did.contains_key(did) {
            return DidSubmissionRetryClass::FinalizedNoRetry;
        }

        DidSubmissionRetryClass::RetryableInFlight
    }

    fn validate_document_did(
        did: &AgentDid,
        document: &DidDocument,
    ) -> Result<(), DidRegistryError> {
        if document.id != did.as_str() {
            return Err(DidRegistryError::DocumentDidMismatch {
                expected: did.as_str().to_owned(),
                actual: document.id.clone(),
            });
        }
        Ok(())
    }

    fn authorize_mutation_actor(
        &self,
        did: &AgentDid,
        actor_did: &str,
    ) -> Result<(), DidRegistryError> {
        let record = self
            .records
            .get(did)
            .ok_or_else(|| DidRegistryError::NotFound(did.as_str().to_owned()))?;
        let required_actor = record
            .document
            .metadata
            .operator
            .clone()
            .unwrap_or_else(|| did.as_str().to_owned());

        if actor_did != required_actor {
            return Err(DidRegistryError::UnauthorizedMutationActor {
                did: did.as_str().to_owned(),
                actor_did: actor_did.to_owned(),
                required_actor,
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// DID registry error taxonomy.
pub enum DidRegistryError {
    /// DID already exists in active state.
    AlreadyRegistered(String),
    /// Finality update conflicts with existing record at same/newer sequence.
    ConflictingFinalityUpdate {
        /// DID identifier.
        did: String,
        /// Sequence where conflict occurred.
        sequence: u64,
    },
    /// Provided idempotency key conflicts with existing key for DID.
    ConflictingSubmissionIdempotencyKey {
        /// DID identifier.
        did: String,
        /// Existing idempotency key recorded by registry.
        existing_key: String,
        /// New idempotency key supplied by caller.
        provided_key: String,
    },
    /// DID not found in registry.
    NotFound(String),
    /// Finality update sequence is older than current sequence.
    StaleFinalityUpdate {
        /// DID identifier.
        did: String,
        /// Current accepted sequence.
        current_sequence: u64,
        /// Attempted stale sequence.
        attempted_sequence: u64,
    },
    /// Finality update references unknown idempotency key.
    UnknownSubmissionIdempotencyKey {
        /// DID identifier.
        did: String,
        /// Unrecognized idempotency key.
        idempotency_key: String,
    },
    /// DID exists but is revoked.
    Revoked(String),
    /// DID in document payload does not match target DID.
    DocumentDidMismatch {
        /// Expected DID identifier.
        expected: String,
        /// DID identifier found in document.
        actual: String,
    },
    /// Mutation nonce is invalid (zero).
    InvalidMutationNonce {
        /// DID identifier.
        did: String,
        /// Invalid nonce value.
        nonce: u64,
    },
    /// Mutation nonce replay/non-monotonic value detected.
    ReplayedMutationNonce {
        /// DID identifier.
        did: String,
        /// Last accepted nonce.
        last_nonce: u64,
        /// Newly provided nonce.
        found: u64,
    },
    /// Actor DID is not authorized to mutate lifecycle state.
    UnauthorizedMutationActor {
        /// DID identifier.
        did: String,
        /// Actor DID provided by caller.
        actor_did: String,
        /// Required authorized actor DID.
        required_actor: String,
    },
    /// Requested lifecycle action is invalid for current state.
    InvalidLifecycleMutationTransition {
        /// DID identifier.
        did: String,
        /// Lifecycle action label.
        action: &'static str,
        /// Revocation state before action.
        from_revoked: bool,
    },
}

impl DidRegistryError {
    /// Returns stable reason code for telemetry/policy contract lanes.
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::AlreadyRegistered(_) => "did_registry_already_registered",
            Self::ConflictingFinalityUpdate { .. } => "did_registry_finality_conflict",
            Self::ConflictingSubmissionIdempotencyKey { .. } => {
                "did_registry_submission_key_conflict"
            }
            Self::NotFound(_) => "did_registry_not_found",
            Self::StaleFinalityUpdate { .. } => "did_registry_finality_stale",
            Self::UnknownSubmissionIdempotencyKey { .. } => "did_registry_submission_key_unknown",
            Self::Revoked(_) => "did_registry_revoked",
            Self::DocumentDidMismatch { .. } => "did_registry_document_did_mismatch",
            Self::InvalidMutationNonce { .. } => "did_lifecycle_mutation_nonce_invalid",
            Self::ReplayedMutationNonce { .. } => "did_lifecycle_mutation_nonce_replay",
            Self::UnauthorizedMutationActor { .. } => "did_lifecycle_mutation_unauthorized_actor",
            Self::InvalidLifecycleMutationTransition { .. } => {
                "did_lifecycle_mutation_invalid_transition"
            }
        }
    }
}

impl fmt::Display for DidRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyRegistered(value) => write!(f, "did is already registered: {value}"),
            Self::ConflictingFinalityUpdate { did, sequence } => write!(
                f,
                "conflicting finality update for did {did} at sequence {sequence}"
            ),
            Self::ConflictingSubmissionIdempotencyKey {
                did,
                existing_key,
                provided_key,
            } => write!(
                f,
                "conflicting submission idempotency key for did {did}; existing {existing_key}, provided {provided_key}"
            ),
            Self::NotFound(value) => write!(f, "did not found: {value}"),
            Self::StaleFinalityUpdate {
                did,
                current_sequence,
                attempted_sequence,
            } => write!(
                f,
                "stale finality update for did {did}; current sequence {current_sequence}, attempted {attempted_sequence}"
            ),
            Self::UnknownSubmissionIdempotencyKey {
                did,
                idempotency_key,
            } => write!(
                f,
                "unknown submission idempotency key for did {did}: {idempotency_key}"
            ),
            Self::Revoked(value) => write!(f, "did is revoked: {value}"),
            Self::DocumentDidMismatch { expected, actual } => {
                write!(
                    f,
                    "did document id mismatch, expected {expected}, got {actual}"
                )
            }
            Self::InvalidMutationNonce { did, nonce } => {
                write!(f, "invalid lifecycle mutation nonce for did {did}: {nonce}")
            }
            Self::ReplayedMutationNonce {
                did,
                last_nonce,
                found,
            } => {
                write!(
                    f,
                    "replayed lifecycle mutation nonce for did {did}; last {last_nonce}, found {found}"
                )
            }
            Self::UnauthorizedMutationActor {
                did,
                actor_did,
                required_actor,
            } => {
                write!(
                    f,
                    "unauthorized lifecycle mutation actor for did {did}; actor {actor_did}, required {required_actor}"
                )
            }
            Self::InvalidLifecycleMutationTransition {
                did,
                action,
                from_revoked,
            } => {
                write!(
                    f,
                    "invalid lifecycle mutation transition for did {did}; action {action}, revoked={from_revoked}"
                )
            }
        }
    }
}

impl std::error::Error for DidRegistryError {}

#[cfg(test)]
mod tests {
    use super::{DidRegistry, DidRegistryError, DidSubmissionRetryClass};
    use crate::{canonical_did_document, AgentDid, AgentDidMetadata};

    fn metadata() -> AgentDidMetadata {
        AgentDidMetadata {
            agent_type: "autonomous".to_owned(),
            model_family: "claude-4".to_owned(),
            capabilities: vec!["text".to_owned()],
            operator: None,
        }
    }

    fn document_for(did: &AgentDid) -> crate::DidDocument {
        canonical_did_document(did, "z6Mpubkey", metadata()).expect("document should build")
    }

    #[test]
    fn rejects_document_mismatch() {
        let mut registry = DidRegistry::new();
        let did = AgentDid::parse("kamn:did:agent:agent-1").expect("did should parse");
        let other = AgentDid::parse("kamn:did:agent:agent-2").expect("did should parse");

        assert_eq!(
            registry.register(did.clone(), document_for(&other)),
            Err(DidRegistryError::DocumentDidMismatch {
                expected: did.as_str().to_owned(),
                actual: other.as_str().to_owned(),
            })
        );
    }

    #[test]
    fn update_rejects_revoked_did() {
        let mut registry = DidRegistry::new();
        let did = AgentDid::parse("kamn:did:agent:agent-3").expect("did should parse");
        registry
            .register(did.clone(), document_for(&did))
            .expect("register should succeed");
        registry.revoke(&did).expect("revoke should succeed");

        assert_eq!(
            registry.update(did.clone(), document_for(&did)),
            Err(DidRegistryError::Revoked(did.as_str().to_owned()))
        );
    }

    #[test]
    fn idempotency_key_generation_is_deterministic() {
        let registry = DidRegistry::new();
        let did = AgentDid::parse("kamn:did:agent:agent-4").expect("did should parse");
        let document = document_for(&did);

        let key_a = registry
            .idempotency_key_for_register(&did, &document)
            .expect("first key should derive");
        let key_b = registry
            .idempotency_key_for_register(&did, &document)
            .expect("second key should derive");

        assert_eq!(key_a, key_b);
    }

    #[test]
    fn retry_classification_rejects_conflicting_document_key() {
        let mut registry = DidRegistry::new();
        let did = AgentDid::parse("kamn:did:agent:agent-5").expect("did should parse");
        let original = document_for(&did);
        let mut changed = document_for(&did);
        changed.metadata.model_family = "gpt-5".to_owned();

        assert_eq!(
            registry
                .register_with_retry_guard(did.clone(), original.clone())
                .expect("first submit should succeed"),
            DidSubmissionRetryClass::NewSubmission
        );
        assert_eq!(
            registry
                .classify_register_retry(&did, &original)
                .expect("duplicate should classify"),
            DidSubmissionRetryClass::RetryableInFlight
        );
        assert_eq!(
            registry
                .classify_register_retry(&did, &changed)
                .expect("changed document should classify"),
            DidSubmissionRetryClass::ConflictNoRetry
        );
    }
}
