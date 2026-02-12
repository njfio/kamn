//! Validator lifecycle state machine and transition-proof policy contracts.
//!
//! This module models validator onboarding, offboarding, quorum reconfiguration,
//! and rollback actions with auditable transition records.

use crate::AgentDid;
use std::collections::BTreeSet;
use std::fmt;

/// Evidence bundle authorizing a validator set transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorTransitionProof {
    /// Governance or workflow proposal identifier for this transition.
    pub proposal_id: String,
    /// Validator DIDs that approved the transition proposal.
    pub approver_dids: Vec<String>,
    /// Deterministic digest for the transition authorization artifact.
    pub proof_hash: String,
}

/// Materialized validator-set state after a transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorSetSnapshot {
    /// Active validator DIDs in deterministic order.
    pub validator_dids: Vec<String>,
    /// Minimum approvals required for future transition proofs.
    pub quorum_threshold: usize,
    /// Unix timestamp when this snapshot became active.
    pub updated_at_unix: u64,
}

/// Supported validator-set transition kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatorTransitionKind {
    /// Adds a new validator DID to the active set.
    Onboard,
    /// Removes an existing validator DID from the active set.
    Offboard,
    /// Changes quorum threshold without changing validator membership.
    ReconfigureQuorum,
    /// Reverts the previous transition and records rollback provenance.
    Rollback,
}

/// Immutable audit record describing one validator-set transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorTransitionRecord {
    /// Transition variant executed.
    pub kind: ValidatorTransitionKind,
    /// Subject validator DID for member add/remove transitions.
    pub subject_validator_did: Option<String>,
    /// Snapshot before applying the transition.
    pub previous_snapshot: ValidatorSetSnapshot,
    /// Snapshot after applying the transition.
    pub next_snapshot: ValidatorSetSnapshot,
    /// Proof material used to authorize the transition.
    pub proof: ValidatorTransitionProof,
    /// Unix timestamp when transition was requested.
    pub requested_at_unix: u64,
}

/// In-memory manager enforcing validator lifecycle transition policies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorLifecycleManager {
    validator_dids: BTreeSet<String>,
    quorum_threshold: usize,
    updated_at_unix: u64,
    transitions: Vec<ValidatorTransitionRecord>,
    consumed_transition_proofs: BTreeSet<String>,
}

impl ValidatorLifecycleManager {
    /// Constructs a manager with an initial validator set and quorum threshold.
    pub fn new(
        validator_dids: Vec<String>,
        quorum_threshold: usize,
    ) -> Result<Self, ValidatorLifecycleError> {
        if validator_dids.is_empty() {
            return Err(ValidatorLifecycleError::EmptyValidatorSet);
        }

        let mut set = BTreeSet::new();
        for did in validator_dids {
            validate_did(&did)?;
            if !set.insert(did.clone()) {
                return Err(ValidatorLifecycleError::DuplicateValidator(did));
            }
        }

        validate_quorum_threshold(quorum_threshold, set.len())?;
        Ok(Self {
            validator_dids: set,
            quorum_threshold,
            updated_at_unix: 0,
            transitions: Vec::new(),
            consumed_transition_proofs: BTreeSet::new(),
        })
    }

    /// Adds a validator after DID/proof validation and self-approval checks.
    pub fn onboard_validator(
        &mut self,
        validator_did: &str,
        proof: &ValidatorTransitionProof,
        requested_at_unix: u64,
    ) -> Result<(), ValidatorLifecycleError> {
        validate_timestamp("requested_at_unix", requested_at_unix)?;
        validate_did(validator_did)?;
        validate_transition_proof(proof, self.quorum_threshold)?;
        reject_onboarding_self_approval(validator_did, proof)?;
        self.ensure_transition_proof_not_replayed(proof)?;
        if self.validator_dids.contains(validator_did) {
            return Err(ValidatorLifecycleError::DuplicateValidator(
                validator_did.to_owned(),
            ));
        }

        let previous_snapshot = self.snapshot();
        self.validator_dids.insert(validator_did.to_owned());
        self.updated_at_unix = requested_at_unix;
        let next_snapshot = self.snapshot();
        self.transitions.push(ValidatorTransitionRecord {
            kind: ValidatorTransitionKind::Onboard,
            subject_validator_did: Some(validator_did.to_owned()),
            previous_snapshot,
            next_snapshot,
            proof: proof.clone(),
            requested_at_unix,
        });
        self.consume_transition_proof(proof);
        Ok(())
    }

    /// Removes a validator while ensuring quorum remains satisfiable.
    pub fn offboard_validator(
        &mut self,
        validator_did: &str,
        proof: &ValidatorTransitionProof,
        requested_at_unix: u64,
    ) -> Result<(), ValidatorLifecycleError> {
        validate_timestamp("requested_at_unix", requested_at_unix)?;
        validate_did(validator_did)?;
        validate_transition_proof(proof, self.quorum_threshold)?;
        self.ensure_transition_proof_not_replayed(proof)?;
        if !self.validator_dids.contains(validator_did) {
            return Err(ValidatorLifecycleError::ValidatorNotFound(
                validator_did.to_owned(),
            ));
        }

        let validator_count_after = self.validator_dids.len().saturating_sub(1);
        if self.quorum_threshold > validator_count_after {
            return Err(ValidatorLifecycleError::QuorumWouldExceedValidatorCount {
                quorum_threshold: self.quorum_threshold,
                validator_count: validator_count_after,
            });
        }

        let previous_snapshot = self.snapshot();
        self.validator_dids.remove(validator_did);
        self.updated_at_unix = requested_at_unix;
        let next_snapshot = self.snapshot();
        self.transitions.push(ValidatorTransitionRecord {
            kind: ValidatorTransitionKind::Offboard,
            subject_validator_did: Some(validator_did.to_owned()),
            previous_snapshot,
            next_snapshot,
            proof: proof.clone(),
            requested_at_unix,
        });
        self.consume_transition_proof(proof);
        Ok(())
    }

    /// Updates quorum threshold using the current validator governance proof.
    pub fn reconfigure_quorum(
        &mut self,
        new_quorum_threshold: usize,
        proof: &ValidatorTransitionProof,
        requested_at_unix: u64,
    ) -> Result<(), ValidatorLifecycleError> {
        validate_timestamp("requested_at_unix", requested_at_unix)?;
        validate_transition_proof(proof, self.quorum_threshold)?;
        self.ensure_transition_proof_not_replayed(proof)?;
        validate_quorum_threshold(new_quorum_threshold, self.validator_dids.len())?;

        let previous_snapshot = self.snapshot();
        self.quorum_threshold = new_quorum_threshold;
        self.updated_at_unix = requested_at_unix;
        let next_snapshot = self.snapshot();
        self.transitions.push(ValidatorTransitionRecord {
            kind: ValidatorTransitionKind::ReconfigureQuorum,
            subject_validator_did: None,
            previous_snapshot,
            next_snapshot,
            proof: proof.clone(),
            requested_at_unix,
        });
        self.consume_transition_proof(proof);
        Ok(())
    }

    /// Rolls back the last transition and appends an explicit rollback record.
    pub fn rollback_last_transition(
        &mut self,
        rolled_back_by: &str,
        reason: &str,
        requested_at_unix: u64,
    ) -> Result<(), ValidatorLifecycleError> {
        validate_timestamp("requested_at_unix", requested_at_unix)?;
        validate_did(rolled_back_by)?;
        require_non_empty("reason", reason)?;

        let Some(last_transition) = self.transitions.pop() else {
            return Err(ValidatorLifecycleError::NoTransitionToRollback);
        };

        self.validator_dids = last_transition
            .previous_snapshot
            .validator_dids
            .iter()
            .cloned()
            .collect();
        self.quorum_threshold = last_transition.previous_snapshot.quorum_threshold;
        self.updated_at_unix = requested_at_unix;
        let previous_snapshot = last_transition.next_snapshot;
        let next_snapshot = self.snapshot();
        self.transitions.push(ValidatorTransitionRecord {
            kind: ValidatorTransitionKind::Rollback,
            subject_validator_did: None,
            previous_snapshot,
            next_snapshot,
            proof: ValidatorTransitionProof {
                proposal_id: format!("rollback:{rolled_back_by}"),
                approver_dids: vec![rolled_back_by.to_owned()],
                proof_hash: reason.to_owned(),
            },
            requested_at_unix,
        });
        Ok(())
    }

    /// Returns the current validator-set snapshot.
    pub fn snapshot(&self) -> ValidatorSetSnapshot {
        ValidatorSetSnapshot {
            validator_dids: self.validator_dids.iter().cloned().collect(),
            quorum_threshold: self.quorum_threshold,
            updated_at_unix: self.updated_at_unix,
        }
    }

    /// Returns transition history in insertion order.
    pub fn transition_history(&self) -> Vec<ValidatorTransitionRecord> {
        self.transitions.clone()
    }

    fn ensure_transition_proof_not_replayed(
        &self,
        proof: &ValidatorTransitionProof,
    ) -> Result<(), ValidatorLifecycleError> {
        if self
            .consumed_transition_proofs
            .contains(&transition_proof_fingerprint(proof))
        {
            return Err(ValidatorLifecycleError::TransitionProofReplay {
                proposal_id: proof.proposal_id.clone(),
                proof_hash: proof.proof_hash.clone(),
            });
        }
        Ok(())
    }

    fn consume_transition_proof(&mut self, proof: &ValidatorTransitionProof) {
        self.consumed_transition_proofs
            .insert(transition_proof_fingerprint(proof));
    }
}

/// Error surface for validator lifecycle state and transition-proof validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatorLifecycleError {
    /// A required string field was empty or whitespace.
    EmptyField(&'static str),
    /// DID value failed `AgentDid` validation.
    InvalidDid(String),
    /// Timestamp field was zero.
    InvalidTimestamp(&'static str),
    /// Initial validator set was empty.
    EmptyValidatorSet,
    /// Validator DID already exists in the active set.
    DuplicateValidator(String),
    /// Validator DID does not exist in the active set.
    ValidatorNotFound(String),
    /// Quorum threshold is outside `1..=validator_count`.
    InvalidQuorumThreshold {
        /// Candidate quorum threshold value.
        quorum_threshold: usize,
        /// Active validator count used for validation.
        validator_count: usize,
    },
    /// Offboarding would violate quorum/validator-count invariants.
    QuorumWouldExceedValidatorCount {
        /// Existing quorum threshold that would become invalid.
        quorum_threshold: usize,
        /// Validator count after the attempted offboarding.
        validator_count: usize,
    },
    /// Transition proof omitted required fields.
    InvalidTransitionProof(&'static str),
    /// Transition proof did not include enough unique approvers.
    InsufficientTransitionApprovals {
        /// Required unique approval count.
        required: usize,
        /// Unique approval count provided by the proof.
        provided: usize,
    },
    /// Transition proof fingerprint has already been consumed.
    TransitionProofReplay {
        /// Proposal identifier from the replayed proof.
        proposal_id: String,
        /// Proof hash from the replayed proof.
        proof_hash: String,
    },
    /// Candidate validator approved its own onboarding proof.
    OnboardingSelfApproval {
        /// Candidate validator DID that self-approved onboarding.
        validator_did: String,
    },
    /// No prior transition exists to roll back.
    NoTransitionToRollback,
}

impl fmt::Display for ValidatorLifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::InvalidTimestamp(field) => write!(f, "timestamp must be > 0: {field}"),
            Self::EmptyValidatorSet => write!(f, "validator set must not be empty"),
            Self::DuplicateValidator(did) => write!(f, "duplicate validator did: {did}"),
            Self::ValidatorNotFound(did) => write!(f, "validator did not found: {did}"),
            Self::InvalidQuorumThreshold {
                quorum_threshold,
                validator_count,
            } => write!(
                f,
                "invalid quorum threshold {quorum_threshold} for validator count {validator_count}"
            ),
            Self::QuorumWouldExceedValidatorCount {
                quorum_threshold,
                validator_count,
            } => write!(
                f,
                "quorum threshold {quorum_threshold} would exceed validator count {validator_count}"
            ),
            Self::InvalidTransitionProof(field) => {
                write!(f, "invalid validator transition proof field: {field}")
            }
            Self::InsufficientTransitionApprovals { required, provided } => write!(
                f,
                "insufficient transition approvals: required {required}, provided {provided}"
            ),
            Self::TransitionProofReplay {
                proposal_id,
                proof_hash,
            } => write!(
                f,
                "transition proof replay detected: proposal_id={proposal_id}, proof_hash={proof_hash}"
            ),
            Self::OnboardingSelfApproval { validator_did } => write!(
                f,
                "onboarding transition proof cannot include candidate self-approval: {validator_did}"
            ),
            Self::NoTransitionToRollback => write!(f, "no validator transition to rollback"),
        }
    }
}

impl std::error::Error for ValidatorLifecycleError {}

fn validate_transition_proof(
    proof: &ValidatorTransitionProof,
    required_approvals: usize,
) -> Result<(), ValidatorLifecycleError> {
    require_non_empty("proposal_id", &proof.proposal_id)
        .map_err(|_| ValidatorLifecycleError::InvalidTransitionProof("proposal_id"))?;
    require_non_empty("proof_hash", &proof.proof_hash)
        .map_err(|_| ValidatorLifecycleError::InvalidTransitionProof("proof_hash"))?;
    if proof.approver_dids.is_empty() {
        return Err(ValidatorLifecycleError::InvalidTransitionProof(
            "approver_dids",
        ));
    }

    let mut unique_approvers = BTreeSet::new();
    for approver in &proof.approver_dids {
        validate_did(approver)?;
        unique_approvers.insert(approver.clone());
    }
    if unique_approvers.len() < required_approvals {
        return Err(ValidatorLifecycleError::InsufficientTransitionApprovals {
            required: required_approvals,
            provided: unique_approvers.len(),
        });
    }

    Ok(())
}

fn reject_onboarding_self_approval(
    validator_did: &str,
    proof: &ValidatorTransitionProof,
) -> Result<(), ValidatorLifecycleError> {
    if proof
        .approver_dids
        .iter()
        .any(|approver| approver == validator_did)
    {
        return Err(ValidatorLifecycleError::OnboardingSelfApproval {
            validator_did: validator_did.to_owned(),
        });
    }
    Ok(())
}

fn transition_proof_fingerprint(proof: &ValidatorTransitionProof) -> String {
    format!("{}|{}", proof.proposal_id, proof.proof_hash)
}

fn validate_quorum_threshold(
    quorum_threshold: usize,
    validator_count: usize,
) -> Result<(), ValidatorLifecycleError> {
    if quorum_threshold == 0 || quorum_threshold > validator_count {
        return Err(ValidatorLifecycleError::InvalidQuorumThreshold {
            quorum_threshold,
            validator_count,
        });
    }
    Ok(())
}

fn validate_did(value: &str) -> Result<(), ValidatorLifecycleError> {
    AgentDid::parse(value)
        .map_err(|error| ValidatorLifecycleError::InvalidDid(error.to_string()))?;
    Ok(())
}

fn validate_timestamp(field: &'static str, value: u64) -> Result<(), ValidatorLifecycleError> {
    if value == 0 {
        return Err(ValidatorLifecycleError::InvalidTimestamp(field));
    }
    Ok(())
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), ValidatorLifecycleError> {
    if value.trim().is_empty() {
        return Err(ValidatorLifecycleError::EmptyField(field));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        ValidatorLifecycleError, ValidatorLifecycleManager, ValidatorTransitionKind,
        ValidatorTransitionProof,
    };

    fn proof() -> ValidatorTransitionProof {
        ValidatorTransitionProof {
            proposal_id: "gov-proof-1".to_owned(),
            approver_dids: vec![
                "kamn:did:agent:validator-1".to_owned(),
                "kamn:did:agent:validator-2".to_owned(),
            ],
            proof_hash: "hash-proof-1".to_owned(),
        }
    }

    #[test]
    fn rollback_restores_previous_snapshot() {
        let mut manager = ValidatorLifecycleManager::new(
            vec![
                "kamn:did:agent:validator-1".to_owned(),
                "kamn:did:agent:validator-2".to_owned(),
            ],
            2,
        )
        .expect("manager should initialize");
        manager
            .onboard_validator("kamn:did:agent:validator-3", &proof(), 101)
            .expect("onboard should pass");
        assert_eq!(manager.snapshot().validator_dids.len(), 3);

        manager
            .rollback_last_transition("kamn:did:agent:validator-1", "rollback", 102)
            .expect("rollback should pass");
        assert_eq!(manager.snapshot().validator_dids.len(), 2);
        assert_eq!(
            manager
                .transition_history()
                .last()
                .map(|record| &record.kind),
            Some(&ValidatorTransitionKind::Rollback)
        );
    }

    #[test]
    fn rollback_without_history_is_rejected() {
        let mut manager =
            ValidatorLifecycleManager::new(vec!["kamn:did:agent:validator-1".to_owned()], 1)
                .expect("manager should initialize");
        assert_eq!(
            manager.rollback_last_transition("kamn:did:agent:validator-1", "rollback", 99),
            Err(ValidatorLifecycleError::NoTransitionToRollback)
        );
    }
}
