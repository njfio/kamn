use super::*;
#[derive(Debug, Clone, PartialEq, Eq)]
/// Proposal candidate.
pub struct ProposalCandidate {
    id: String,
    sender_did: String,
    nonce: u64,
    state_hash: String,
}

impl ProposalCandidate {
    /// Handles new.
    pub fn new(
        id: &str,
        sender_did: &str,
        nonce: u64,
        state_hash: &str,
    ) -> Result<Self, ProposalPlannerError> {
        if id.trim().is_empty() {
            return Err(ProposalPlannerError::InvalidCandidateId);
        }
        if sender_did.trim().is_empty() {
            return Err(ProposalPlannerError::InvalidSenderDid);
        }
        if state_hash.trim().is_empty() {
            return Err(ProposalPlannerError::InvalidStateHash);
        }
        if nonce == 0 {
            return Err(ProposalPlannerError::InvalidNonce);
        }
        Ok(Self {
            id: id.to_owned(),
            sender_did: sender_did.to_owned(),
            nonce,
            state_hash: state_hash.to_owned(),
        })
    }

    /// Handles id.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Handles sender did.
    pub fn sender_did(&self) -> &str {
        &self.sender_did
    }

    /// Handles nonce.
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    /// Handles state hash.
    pub fn state_hash(&self) -> &str {
        &self.state_hash
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Proposal plan.
pub struct ProposalPlan {
    ordered_candidates: Vec<ProposalCandidate>,
}

impl ProposalPlan {
    /// Handles ordered candidates.
    pub fn ordered_candidates(&self) -> &[ProposalCandidate] {
        &self.ordered_candidates
    }

    /// Handles ordered candidate ids.
    pub fn ordered_candidate_ids(&self) -> Vec<String> {
        self.ordered_candidates
            .iter()
            .map(|candidate| candidate.id.clone())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Proposal planner error.
pub enum ProposalPlannerError {
    /// Invalid candidate id.
    InvalidCandidateId,
    /// Invalid sender did.
    InvalidSenderDid,
    /// Invalid state hash.
    InvalidStateHash,
    /// Invalid nonce.
    InvalidNonce,
    /// Duplicate candidate id.
    DuplicateCandidateId(String),
    /// Stale state hash.
    StaleStateHash {
        /// Expected state hash.
        expected: String,
        /// Observed state hash.
        found: String,
    },
}

impl Display for ProposalPlannerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCandidateId => write!(f, "proposal candidate id cannot be empty"),
            Self::InvalidSenderDid => write!(f, "proposal candidate sender DID cannot be empty"),
            Self::InvalidStateHash => write!(f, "proposal candidate state hash cannot be empty"),
            Self::InvalidNonce => write!(f, "proposal candidate nonce must be positive"),
            Self::DuplicateCandidateId(id) => {
                write!(f, "duplicate proposal candidate id: {id}")
            }
            Self::StaleStateHash { expected, found } => {
                write!(
                    f,
                    "proposal candidate state hash mismatch: expected {expected}, found {found}"
                )
            }
        }
    }
}

impl Error for ProposalPlannerError {}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Deterministic proposal planner.
pub struct DeterministicProposalPlanner {
    expected_state_hash: String,
}

impl DeterministicProposalPlanner {
    /// Handles new.
    pub fn new(expected_state_hash: &str) -> Self {
        Self {
            expected_state_hash: expected_state_hash.to_owned(),
        }
    }

    /// Handles plan.
    pub fn plan(
        &self,
        mut candidates: Vec<ProposalCandidate>,
    ) -> Result<ProposalPlan, ProposalPlannerError> {
        validate_candidates(&candidates, &self.expected_state_hash)?;
        sort_candidates(&mut candidates);
        Ok(ProposalPlan {
            ordered_candidates: candidates,
        })
    }
}

fn validate_candidates(
    candidates: &[ProposalCandidate],
    expected_state_hash: &str,
) -> Result<(), ProposalPlannerError> {
    let mut seen_ids = HashSet::new();
    for candidate in candidates {
        if !seen_ids.insert(candidate.id.as_str()) {
            return Err(ProposalPlannerError::DuplicateCandidateId(
                candidate.id.clone(),
            ));
        }
        if candidate.state_hash != expected_state_hash {
            return Err(ProposalPlannerError::StaleStateHash {
                expected: expected_state_hash.to_owned(),
                found: candidate.state_hash.clone(),
            });
        }
    }
    Ok(())
}

fn sort_candidates(candidates: &mut [ProposalCandidate]) {
    candidates.sort_by(|left, right| {
        left.nonce
            .cmp(&right.nonce)
            .then_with(|| left.sender_did.cmp(&right.sender_did))
            .then_with(|| left.id.cmp(&right.id))
    });
}
