use super::{BlockPipelineError, CanonicalCommitRecord};

/// Fork-choice decision for canonical candidate commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForkChoiceDecision {
    /// Candidate is accepted as canonical.
    Accept,
    /// Candidate is rejected with deterministic reason code.
    Reject {
        /// Deterministic rejection reason code.
        reason_code: String,
    },
}

/// Fork-choice evaluation hook for candidate canonical commit records.
pub trait ForkChoiceHook {
    /// Evaluates candidate canonical commit and returns deterministic decision.
    fn evaluate_candidate(
        &mut self,
        record: &CanonicalCommitRecord,
    ) -> Result<ForkChoiceDecision, BlockPipelineError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Fork-choice hook that accepts every candidate.
pub struct AcceptAllForkChoiceHook;

impl ForkChoiceHook for AcceptAllForkChoiceHook {
    fn evaluate_candidate(
        &mut self,
        _record: &CanonicalCommitRecord,
    ) -> Result<ForkChoiceDecision, BlockPipelineError> {
        Ok(ForkChoiceDecision::Accept)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// Deterministic fork-choice hook for competing branch candidates.
///
/// Selection rules:
/// - Higher `block_height` always wins.
/// - At equal `block_height`, lexicographically lower `payload_digest` wins.
/// - Identical height+digest is treated as duplicate candidate and rejected.
pub struct DeterministicCompetingBranchForkChoiceHook {
    canonical_head: Option<CanonicalCommitRecord>,
}

impl DeterministicCompetingBranchForkChoiceHook {
    /// Creates deterministic competing-branch hook with empty head.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates deterministic competing-branch hook seeded with existing canonical head.
    pub fn with_canonical_head(canonical_head: CanonicalCommitRecord) -> Self {
        Self {
            canonical_head: Some(canonical_head),
        }
    }

    /// Returns currently selected canonical head, if any.
    pub fn canonical_head(&self) -> Option<&CanonicalCommitRecord> {
        self.canonical_head.as_ref()
    }
}

impl ForkChoiceHook for DeterministicCompetingBranchForkChoiceHook {
    fn evaluate_candidate(
        &mut self,
        record: &CanonicalCommitRecord,
    ) -> Result<ForkChoiceDecision, BlockPipelineError> {
        let Some(head) = self.canonical_head.as_ref() else {
            self.canonical_head = Some(record.clone());
            return Ok(ForkChoiceDecision::Accept);
        };

        if record.block_height > head.block_height {
            self.canonical_head = Some(record.clone());
            return Ok(ForkChoiceDecision::Accept);
        }
        if record.block_height < head.block_height {
            return Ok(ForkChoiceDecision::Reject {
                reason_code: "fork_choice_stale_block_height".to_owned(),
            });
        }

        if record.payload_digest == head.payload_digest {
            return Ok(ForkChoiceDecision::Reject {
                reason_code: "fork_choice_duplicate_candidate".to_owned(),
            });
        }

        if record.payload_digest < head.payload_digest {
            self.canonical_head = Some(record.clone());
            return Ok(ForkChoiceDecision::Accept);
        }

        Ok(ForkChoiceDecision::Reject {
            reason_code: "fork_choice_tie_break_loser".to_owned(),
        })
    }
}
