use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Handles listener DID format validation.
pub(crate) fn is_valid_listener_did(value: &str) -> bool {
    is_valid_kamn_did(value)
}

/// Handles runtime DID format validation.
pub(crate) fn is_valid_kamn_did(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed.starts_with("kamn:did:")
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Rejoin attempt.
pub struct RejoinAttempt {
    node_id: String,
    state_version: u64,
    state_hash: String,
    resume_token: String,
}

impl RejoinAttempt {
    /// Handles new.
    pub fn new(
        node_id: &str,
        state_version: u64,
        state_hash: &str,
        resume_token: &str,
    ) -> Result<Self, RecoveryGuardError> {
        if node_id.trim().is_empty() {
            return Err(RecoveryGuardError::InvalidNodeId);
        }
        if state_version == 0 {
            return Err(RecoveryGuardError::InvalidStateVersion);
        }
        if state_hash.trim().is_empty() {
            return Err(RecoveryGuardError::InvalidStateHash);
        }
        if resume_token.trim().is_empty() {
            return Err(RecoveryGuardError::InvalidResumeToken);
        }
        Ok(Self {
            node_id: node_id.to_owned(),
            state_version,
            state_hash: state_hash.to_owned(),
            resume_token: resume_token.to_owned(),
        })
    }

    /// Handles node id.
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Handles state version.
    pub fn state_version(&self) -> u64 {
        self.state_version
    }

    /// Handles state hash.
    pub fn state_hash(&self) -> &str {
        &self.state_hash
    }

    /// Handles resume token.
    pub fn resume_token(&self) -> &str {
        &self.resume_token
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Recovery status.
pub enum RecoveryStatus {
    /// Rejoin accepted.
    RejoinAccepted,
    /// Catch up required.
    CatchUpRequired {
        /// Current local state version.
        from_version: u64,
        /// Target remote state version.
        to_version: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Recovery guard error.
pub enum RecoveryGuardError {
    /// Invalid node id.
    InvalidNodeId,
    /// Invalid state version.
    InvalidStateVersion,
    /// Invalid state hash.
    InvalidStateHash,
    /// Invalid resume token.
    InvalidResumeToken,
    /// Replay resume token.
    ReplayResumeToken(String),
    /// State version mismatch.
    StateVersionMismatch {
        /// Expected state version.
        expected: u64,
        /// Observed state version.
        found: u64,
    },
    /// State hash mismatch.
    StateHashMismatch {
        /// Expected state hash.
        expected: String,
        /// Observed state hash.
        found: String,
    },
}

impl Display for RecoveryGuardError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidNodeId => write!(f, "rejoin node id cannot be empty"),
            Self::InvalidStateVersion => write!(f, "rejoin state version must be positive"),
            Self::InvalidStateHash => write!(f, "rejoin state hash cannot be empty"),
            Self::InvalidResumeToken => write!(f, "rejoin resume token cannot be empty"),
            Self::ReplayResumeToken(token) => {
                write!(f, "rejoin resume token replayed: {token}")
            }
            Self::StateVersionMismatch { expected, found } => {
                write!(
                    f,
                    "rejoin state version mismatch: expected {expected}, found {found}"
                )
            }
            Self::StateHashMismatch { expected, found } => {
                write!(
                    f,
                    "rejoin state hash mismatch: expected {expected}, found {found}"
                )
            }
        }
    }
}

impl Error for RecoveryGuardError {}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Recovery rejoin guard.
pub struct RecoveryRejoinGuard {
    expected_state_version: u64,
    expected_state_hash: String,
    consumed_resume_tokens: HashSet<String>,
}

impl RecoveryRejoinGuard {
    /// Handles new.
    pub fn new(
        expected_state_version: u64,
        expected_state_hash: &str,
    ) -> Result<Self, RecoveryGuardError> {
        if expected_state_version == 0 {
            return Err(RecoveryGuardError::InvalidStateVersion);
        }
        if expected_state_hash.trim().is_empty() {
            return Err(RecoveryGuardError::InvalidStateHash);
        }
        Ok(Self {
            expected_state_version,
            expected_state_hash: expected_state_hash.to_owned(),
            consumed_resume_tokens: HashSet::new(),
        })
    }

    /// Handles evaluate.
    pub fn evaluate(
        &mut self,
        attempt: RejoinAttempt,
    ) -> Result<RecoveryStatus, RecoveryGuardError> {
        if self.consumed_resume_tokens.contains(attempt.resume_token()) {
            return Err(RecoveryGuardError::ReplayResumeToken(
                attempt.resume_token.clone(),
            ));
        }

        if attempt.state_version < self.expected_state_version {
            return Ok(RecoveryStatus::CatchUpRequired {
                from_version: attempt.state_version,
                to_version: self.expected_state_version,
            });
        }

        if attempt.state_version > self.expected_state_version {
            return Err(RecoveryGuardError::StateVersionMismatch {
                expected: self.expected_state_version,
                found: attempt.state_version,
            });
        }

        if attempt.state_hash != self.expected_state_hash {
            return Err(RecoveryGuardError::StateHashMismatch {
                expected: self.expected_state_hash.clone(),
                found: attempt.state_hash,
            });
        }

        self.consumed_resume_tokens.insert(attempt.resume_token);
        Ok(RecoveryStatus::RejoinAccepted)
    }
}
