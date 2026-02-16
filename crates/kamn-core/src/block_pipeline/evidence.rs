use super::{BlockPipelineError, CanonicalCommitRecord};
use std::collections::BTreeSet;

const TRANSPORT_CONVERGENCE_EVIDENCE_SCHEMA_VERSION: &str =
    "kamn.runtime.transport-convergence-evidence.v1";
const CANONICAL_REPLAY_EVIDENCE_SCHEMA_VERSION: &str = "kamn.runtime.canonical-replay-evidence.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
/// Restart/replay evidence bundle for canonical commit lineage continuity.
pub struct CanonicalReplayEvidenceBundle {
    /// Versioned schema marker for policy and contract-lane checks.
    pub schema_version: String,
    /// Canonical head block height at restart boundary.
    pub restart_boundary_block_height: u64,
    /// Canonical head block height after replay checkpoint validation.
    pub replay_checkpoint_block_height: u64,
    /// Canonical commit count captured before restart.
    pub pre_restart_commit_count: usize,
    /// Canonical commit count observed after restart/replay.
    pub post_restart_commit_count: usize,
    /// Deterministic continuity status marker (`verified` on success).
    pub continuity_status: String,
}

/// Deterministic decision outcome for one transport candidate reconciliation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalCandidateDecision {
    /// Candidate was selected as canonical and persisted.
    Accepted,
    /// Candidate was rejected by fork-choice with deterministic reason code.
    Rejected {
        /// Deterministic fork-choice rejection reason code.
        reason_code: String,
    },
}

/// Reconciliation report for one transport-provided canonical candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalCandidateOutcome {
    /// Candidate block height.
    pub block_height: u64,
    /// Candidate payload digest.
    pub payload_digest: String,
    /// Deterministic reconciliation decision.
    pub decision: CanonicalCandidateDecision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Evidence bundle summarizing one transport convergence fault-drill execution.
pub struct TransportConvergenceEvidenceBundle {
    /// Versioned schema marker for policy and contract-lane checks.
    pub schema_version: String,
    /// Fault-drill case identifier.
    pub case_id: String,
    /// Count of accepted canonical candidates.
    pub accepted_candidate_count: usize,
    /// Count of rejected canonical candidates.
    pub rejected_candidate_count: usize,
    /// Deterministic set of rejection reason codes observed in this case.
    pub rejected_reason_codes: Vec<String>,
    /// Persisted canonical commit count after reconciliation.
    pub persisted_commit_count: usize,
    /// Highest persisted canonical block height after reconciliation.
    pub persisted_highest_block_height: Option<u64>,
    /// Deterministic continuity status marker (`verified` on success).
    pub continuity_status: String,
}

/// Builds deterministic convergence evidence for partition/rejoin and publish-drop drills.
pub fn build_transport_convergence_evidence_bundle(
    case_id: &str,
    outcomes: &[CanonicalCandidateOutcome],
    persisted_commits: &[CanonicalCommitRecord],
) -> Result<TransportConvergenceEvidenceBundle, BlockPipelineError> {
    if case_id.trim().is_empty() {
        return Err(BlockPipelineError::ReplayDrift {
            reason_code: "transport_convergence_case_id_missing".to_owned(),
            detail: "transport convergence case id cannot be empty".to_owned(),
        });
    }

    let mut accepted_candidate_count = 0usize;
    let mut rejected_candidate_count = 0usize;
    let mut rejected_reason_codes = BTreeSet::new();
    for outcome in outcomes {
        match &outcome.decision {
            CanonicalCandidateDecision::Accepted => {
                accepted_candidate_count += 1;
            }
            CanonicalCandidateDecision::Rejected { reason_code } => {
                rejected_candidate_count += 1;
                rejected_reason_codes.insert(reason_code.clone());
            }
        }
    }

    let mut highest = None;
    for record in persisted_commits {
        if let Some(previous) = highest {
            if record.block_height <= previous {
                return Err(BlockPipelineError::ReplayDrift {
                    reason_code: "transport_convergence_commit_height_regression".to_owned(),
                    detail: format!(
                        "persisted convergence commit height regression: previous {previous}, found {}",
                        record.block_height
                    ),
                });
            }
        }
        highest = Some(record.block_height);
    }

    Ok(TransportConvergenceEvidenceBundle {
        schema_version: TRANSPORT_CONVERGENCE_EVIDENCE_SCHEMA_VERSION.to_owned(),
        case_id: case_id.to_owned(),
        accepted_candidate_count,
        rejected_candidate_count,
        rejected_reason_codes: rejected_reason_codes.into_iter().collect(),
        persisted_commit_count: persisted_commits.len(),
        persisted_highest_block_height: highest,
        continuity_status: "verified".to_owned(),
    })
}

/// Validates canonical lineage continuity across restart/replay checkpoints.
pub fn build_canonical_replay_evidence_bundle(
    pre_restart: &[CanonicalCommitRecord],
    post_restart: &[CanonicalCommitRecord],
) -> Result<CanonicalReplayEvidenceBundle, BlockPipelineError> {
    let Some(restart_boundary) = pre_restart.last() else {
        return Err(BlockPipelineError::ReplayDrift {
            reason_code: "canonical_replay_pre_restart_lineage_empty".to_owned(),
            detail: "pre-restart canonical lineage cannot be empty".to_owned(),
        });
    };

    for (index, expected) in pre_restart.iter().enumerate() {
        let Some(found) = post_restart.get(index) else {
            return Err(BlockPipelineError::ReplayDrift {
                reason_code: "canonical_replay_checkpoint_missing".to_owned(),
                detail: format!(
                    "post-restart lineage missing checkpoint at index {index} (expected height {})",
                    expected.block_height
                ),
            });
        };

        if found.block_height != expected.block_height {
            return Err(BlockPipelineError::ReplayDrift {
                reason_code: "canonical_replay_block_height_mismatch".to_owned(),
                detail: format!(
                    "canonical replay block height mismatch at index {index}: expected {}, found {}",
                    expected.block_height, found.block_height
                ),
            });
        }
        if found.producer_role != expected.producer_role {
            return Err(BlockPipelineError::ReplayDrift {
                reason_code: "canonical_replay_producer_role_mismatch".to_owned(),
                detail: format!(
                    "canonical replay producer role mismatch at index {index}: expected {}, found {}",
                    expected.producer_role.as_str(),
                    found.producer_role.as_str()
                ),
            });
        }
        if found.payload_digest != expected.payload_digest {
            return Err(BlockPipelineError::ReplayDrift {
                reason_code: "canonical_replay_payload_digest_mismatch".to_owned(),
                detail: format!(
                    "canonical replay payload digest mismatch at index {index}: expected {}, found {}",
                    expected.payload_digest, found.payload_digest
                ),
            });
        }
        if found.transaction_ids != expected.transaction_ids {
            return Err(BlockPipelineError::ReplayDrift {
                reason_code: "canonical_replay_transaction_ids_mismatch".to_owned(),
                detail: format!(
                    "canonical replay transaction ids mismatch at index {index}: expected {:?}, found {:?}",
                    expected.transaction_ids, found.transaction_ids
                ),
            });
        }
    }

    let replay_checkpoint = post_restart
        .get(pre_restart.len().saturating_sub(1))
        .ok_or_else(|| BlockPipelineError::ReplayDrift {
            reason_code: "canonical_replay_checkpoint_missing".to_owned(),
            detail: "post-restart lineage missing replay checkpoint".to_owned(),
        })?;

    Ok(CanonicalReplayEvidenceBundle {
        schema_version: CANONICAL_REPLAY_EVIDENCE_SCHEMA_VERSION.to_owned(),
        restart_boundary_block_height: restart_boundary.block_height,
        replay_checkpoint_block_height: replay_checkpoint.block_height,
        pre_restart_commit_count: pre_restart.len(),
        post_restart_commit_count: post_restart.len(),
        continuity_status: "verified".to_owned(),
    })
}
