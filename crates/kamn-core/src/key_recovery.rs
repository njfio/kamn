//! Recovery-state workflow for compromised agent signing keys.

use std::collections::HashSet;

/// Lifecycle state for key compromise and recovery operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryState {
    /// The active key can be used for normal signing operations.
    Active,
    /// The active key is marked compromised and cannot be trusted.
    Compromised,
    /// The compromised key has been revoked and recovery can begin.
    Revoked,
    /// A recovery proposal is in progress and waiting for approvals.
    RecoveryPending,
}

/// Error surfaced by key recovery lifecycle operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryError {
    /// A required key identifier was empty.
    EmptyKeyId,
    /// Recovery approval requirements are invalid for the configured approvers.
    InvalidRequiredApprovals {
        /// Requested threshold of required approvals.
        required: usize,
        /// Number of configured approvers available.
        approver_count: usize,
    },
    /// Attempted transition is invalid for the current recovery state.
    InvalidTransition {
        /// Current state where the transition was attempted.
        from: RecoveryState,
        /// Operation name that triggered the invalid transition.
        action: &'static str,
    },
    /// Recovery action was attempted by an untrusted approver.
    UnauthorizedApprover(String),
    /// Finalization attempted without enough distinct approvals.
    InsufficientApprovals {
        /// Required approval threshold.
        required: usize,
        /// Number of approvals collected so far.
        actual: usize,
    },
    /// Proposed key is known to be compromised.
    CompromisedKeyReuse(String),
    /// Recovery nonce was already used by a prior finalized proposal.
    ReplayNonce(u64),
    /// The same approver attempted to approve twice.
    DuplicateApproval(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecoveryProposal {
    candidate_key: String,
    nonce: u64,
    approvals: HashSet<String>,
}

/// Manages emergency compromise, revocation, and multi-approver recovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRecoveryManager {
    state: RecoveryState,
    current_key_id: String,
    compromised_keys: HashSet<String>,
    authorized_approvers: HashSet<String>,
    required_approvals: usize,
    used_nonces: HashSet<u64>,
    proposal: Option<RecoveryProposal>,
}

impl KeyRecoveryManager {
    /// Creates a new recovery manager for the active key and approver set.
    pub fn new(
        current_key_id: &str,
        authorized_approvers: Vec<String>,
        required_approvals: usize,
    ) -> Result<Self, RecoveryError> {
        if current_key_id.trim().is_empty() {
            return Err(RecoveryError::EmptyKeyId);
        }
        let approver_count = authorized_approvers.len();
        if required_approvals == 0 || required_approvals > approver_count {
            return Err(RecoveryError::InvalidRequiredApprovals {
                required: required_approvals,
                approver_count,
            });
        }

        Ok(Self {
            state: RecoveryState::Active,
            current_key_id: current_key_id.to_owned(),
            compromised_keys: HashSet::new(),
            authorized_approvers: authorized_approvers.into_iter().collect(),
            required_approvals,
            used_nonces: HashSet::new(),
            proposal: None,
        })
    }

    /// Returns the current recovery lifecycle state.
    pub fn state(&self) -> RecoveryState {
        self.state
    }

    /// Returns the currently active key identifier.
    pub fn current_key_id(&self) -> &str {
        &self.current_key_id
    }

    /// Marks the active key as compromised and enters compromised state.
    pub fn declare_compromised(&mut self, _reason: &str) -> Result<(), RecoveryError> {
        if self.state != RecoveryState::Active {
            return Err(RecoveryError::InvalidTransition {
                from: self.state,
                action: "declare_compromised",
            });
        }

        self.compromised_keys.insert(self.current_key_id.clone());
        self.state = RecoveryState::Compromised;
        Ok(())
    }

    /// Revokes the compromised key and opens recovery proposal flow.
    pub fn emergency_revoke(&mut self) -> Result<(), RecoveryError> {
        if self.state != RecoveryState::Compromised {
            return Err(RecoveryError::InvalidTransition {
                from: self.state,
                action: "emergency_revoke",
            });
        }
        self.state = RecoveryState::Revoked;
        Ok(())
    }

    /// Starts a recovery proposal with the first approver signature.
    pub fn propose_recovery(
        &mut self,
        candidate_key_id: &str,
        proposer: &str,
        nonce: u64,
    ) -> Result<(), RecoveryError> {
        if self.state != RecoveryState::Revoked {
            return Err(RecoveryError::InvalidTransition {
                from: self.state,
                action: "propose_recovery",
            });
        }
        if candidate_key_id.trim().is_empty() {
            return Err(RecoveryError::EmptyKeyId);
        }
        if self.compromised_keys.contains(candidate_key_id) {
            return Err(RecoveryError::CompromisedKeyReuse(
                candidate_key_id.to_owned(),
            ));
        }
        if self.used_nonces.contains(&nonce) {
            return Err(RecoveryError::ReplayNonce(nonce));
        }
        if !self.authorized_approvers.contains(proposer) {
            return Err(RecoveryError::UnauthorizedApprover(proposer.to_owned()));
        }

        let mut approvals = HashSet::new();
        approvals.insert(proposer.to_owned());
        self.proposal = Some(RecoveryProposal {
            candidate_key: candidate_key_id.to_owned(),
            nonce,
            approvals,
        });
        self.state = RecoveryState::RecoveryPending;
        Ok(())
    }

    /// Adds an approver vote to the in-flight recovery proposal.
    pub fn approve_recovery(&mut self, approver: &str) -> Result<(), RecoveryError> {
        if self.state != RecoveryState::RecoveryPending {
            return Err(RecoveryError::InvalidTransition {
                from: self.state,
                action: "approve_recovery",
            });
        }
        if !self.authorized_approvers.contains(approver) {
            return Err(RecoveryError::UnauthorizedApprover(approver.to_owned()));
        }

        let proposal = match self.proposal.as_mut() {
            Some(value) => value,
            None => {
                return Err(RecoveryError::InvalidTransition {
                    from: self.state,
                    action: "approve_recovery",
                });
            }
        };
        if !proposal.approvals.insert(approver.to_owned()) {
            return Err(RecoveryError::DuplicateApproval(approver.to_owned()));
        }
        Ok(())
    }

    /// Finalizes recovery and activates the proposed key once threshold is met.
    pub fn finalize_recovery(&mut self) -> Result<(), RecoveryError> {
        if self.state != RecoveryState::RecoveryPending {
            return Err(RecoveryError::InvalidTransition {
                from: self.state,
                action: "finalize_recovery",
            });
        }
        let proposal = match self.proposal.take() {
            Some(value) => value,
            None => {
                return Err(RecoveryError::InvalidTransition {
                    from: self.state,
                    action: "finalize_recovery",
                });
            }
        };
        let approval_count = proposal.approvals.len();
        if approval_count < self.required_approvals {
            self.proposal = Some(proposal);
            return Err(RecoveryError::InsufficientApprovals {
                required: self.required_approvals,
                actual: approval_count,
            });
        }

        self.used_nonces.insert(proposal.nonce);
        self.current_key_id = proposal.candidate_key;
        self.state = RecoveryState::Active;
        Ok(())
    }

    /// Rejects usage of keys that are recorded as compromised.
    pub fn verify_key_use(&self, key_id: &str) -> Result<(), RecoveryError> {
        if self.compromised_keys.contains(key_id) {
            return Err(RecoveryError::CompromisedKeyReuse(key_id.to_owned()));
        }
        Ok(())
    }
}

impl std::fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyKeyId => write!(f, "key id must not be empty"),
            Self::InvalidRequiredApprovals {
                required,
                approver_count,
            } => write!(
                f,
                "invalid required approvals {required}, approver count {approver_count}"
            ),
            Self::InvalidTransition { from, action } => {
                write!(f, "invalid recovery transition from {from:?} via {action}")
            }
            Self::UnauthorizedApprover(value) => write!(f, "unauthorized approver: {value}"),
            Self::InsufficientApprovals { required, actual } => {
                write!(
                    f,
                    "insufficient approvals, required {required}, actual {actual}"
                )
            }
            Self::CompromisedKeyReuse(value) => write!(f, "compromised key reuse: {value}"),
            Self::ReplayNonce(value) => write!(f, "replay nonce rejected: {value}"),
            Self::DuplicateApproval(value) => write!(f, "duplicate approval: {value}"),
        }
    }
}

impl std::error::Error for RecoveryError {}

#[cfg(test)]
mod tests {
    use super::{KeyRecoveryManager, RecoveryError, RecoveryState};

    #[test]
    fn constructor_rejects_invalid_required_approvals() {
        assert_eq!(
            KeyRecoveryManager::new("key_v1", vec!["a".to_owned()], 2),
            Err(RecoveryError::InvalidRequiredApprovals {
                required: 2,
                approver_count: 1,
            })
        );
    }

    #[test]
    fn duplicate_approval_is_rejected() {
        let mut manager = match KeyRecoveryManager::new(
            "key_v1",
            vec!["approver_a".to_owned(), "approver_b".to_owned()],
            2,
        ) {
            Ok(value) => value,
            Err(error) => panic!("init failed: {error}"),
        };
        if let Err(error) = manager.declare_compromised("leak") {
            panic!("declare failed: {error}");
        }
        if let Err(error) = manager.emergency_revoke() {
            panic!("revoke failed: {error}");
        }
        if let Err(error) = manager.propose_recovery("key_v2", "approver_a", 12) {
            panic!("proposal failed: {error}");
        }
        assert_eq!(
            manager.approve_recovery("approver_a"),
            Err(RecoveryError::DuplicateApproval("approver_a".to_owned()))
        );
        assert_eq!(manager.state(), RecoveryState::RecoveryPending);
    }
}
