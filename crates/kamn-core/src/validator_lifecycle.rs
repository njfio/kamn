use crate::AgentDid;
use std::collections::BTreeSet;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorTransitionProof {
    pub proposal_id: String,
    pub approver_dids: Vec<String>,
    pub proof_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorSetSnapshot {
    pub validator_dids: Vec<String>,
    pub quorum_threshold: usize,
    pub updated_at_unix: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatorTransitionKind {
    Onboard,
    Offboard,
    ReconfigureQuorum,
    Rollback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorTransitionRecord {
    pub kind: ValidatorTransitionKind,
    pub subject_validator_did: Option<String>,
    pub previous_snapshot: ValidatorSetSnapshot,
    pub next_snapshot: ValidatorSetSnapshot,
    pub proof: ValidatorTransitionProof,
    pub requested_at_unix: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorLifecycleManager {
    validator_dids: BTreeSet<String>,
    quorum_threshold: usize,
    updated_at_unix: u64,
    transitions: Vec<ValidatorTransitionRecord>,
}

impl ValidatorLifecycleManager {
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
        })
    }

    pub fn onboard_validator(
        &mut self,
        validator_did: &str,
        proof: &ValidatorTransitionProof,
        requested_at_unix: u64,
    ) -> Result<(), ValidatorLifecycleError> {
        validate_timestamp("requested_at_unix", requested_at_unix)?;
        validate_did(validator_did)?;
        validate_transition_proof(proof, self.quorum_threshold)?;
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
        Ok(())
    }

    pub fn offboard_validator(
        &mut self,
        validator_did: &str,
        proof: &ValidatorTransitionProof,
        requested_at_unix: u64,
    ) -> Result<(), ValidatorLifecycleError> {
        validate_timestamp("requested_at_unix", requested_at_unix)?;
        validate_did(validator_did)?;
        validate_transition_proof(proof, self.quorum_threshold)?;
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
        Ok(())
    }

    pub fn reconfigure_quorum(
        &mut self,
        new_quorum_threshold: usize,
        proof: &ValidatorTransitionProof,
        requested_at_unix: u64,
    ) -> Result<(), ValidatorLifecycleError> {
        validate_timestamp("requested_at_unix", requested_at_unix)?;
        validate_transition_proof(proof, self.quorum_threshold)?;
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
        Ok(())
    }

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

    pub fn snapshot(&self) -> ValidatorSetSnapshot {
        ValidatorSetSnapshot {
            validator_dids: self.validator_dids.iter().cloned().collect(),
            quorum_threshold: self.quorum_threshold,
            updated_at_unix: self.updated_at_unix,
        }
    }

    pub fn transition_history(&self) -> Vec<ValidatorTransitionRecord> {
        self.transitions.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatorLifecycleError {
    EmptyField(&'static str),
    InvalidDid(String),
    InvalidTimestamp(&'static str),
    EmptyValidatorSet,
    DuplicateValidator(String),
    ValidatorNotFound(String),
    InvalidQuorumThreshold {
        quorum_threshold: usize,
        validator_count: usize,
    },
    QuorumWouldExceedValidatorCount {
        quorum_threshold: usize,
        validator_count: usize,
    },
    InvalidTransitionProof(&'static str),
    InsufficientTransitionApprovals {
        required: usize,
        provided: usize,
    },
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
