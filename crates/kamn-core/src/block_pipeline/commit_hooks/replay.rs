use crate::block_pipeline::block_pipeline_support::{
    CanonicalCommitRecord, CanonicalReplayEvidenceBundle,
};
use crate::block_pipeline::models::BlockPipelineError;

const CANONICAL_REPLAY_EVIDENCE_SCHEMA_VERSION: &str = "kamn.runtime.canonical-replay-evidence.v1";

/// Builds restart/replay continuity evidence from pre- and post-restart lineage snapshots.
pub fn build_canonical_replay_evidence_bundle(
    pre_restart: &[CanonicalCommitRecord],
    post_restart: &[CanonicalCommitRecord],
) -> Result<CanonicalReplayEvidenceBundle, BlockPipelineError> {
    let restart_boundary = restart_boundary(pre_restart)?;
    validate_replay_lineage(pre_restart, post_restart)?;
    let replay_checkpoint = replay_checkpoint(pre_restart, post_restart)?;

    Ok(CanonicalReplayEvidenceBundle {
        schema_version: CANONICAL_REPLAY_EVIDENCE_SCHEMA_VERSION.to_owned(),
        restart_boundary_block_height: restart_boundary.block_height,
        replay_checkpoint_block_height: replay_checkpoint.block_height,
        pre_restart_commit_count: pre_restart.len(),
        post_restart_commit_count: post_restart.len(),
        continuity_status: "verified".to_owned(),
    })
}

fn restart_boundary(
    pre_restart: &[CanonicalCommitRecord],
) -> Result<&CanonicalCommitRecord, BlockPipelineError> {
    pre_restart
        .last()
        .ok_or_else(|| BlockPipelineError::ReplayDrift {
            reason_code: "canonical_replay_pre_restart_lineage_empty".to_owned(),
            detail: "pre-restart canonical lineage cannot be empty".to_owned(),
        })
}

fn replay_checkpoint<'a>(
    pre_restart: &[CanonicalCommitRecord],
    post_restart: &'a [CanonicalCommitRecord],
) -> Result<&'a CanonicalCommitRecord, BlockPipelineError> {
    post_restart
        .get(pre_restart.len().saturating_sub(1))
        .ok_or_else(|| BlockPipelineError::ReplayDrift {
            reason_code: "canonical_replay_checkpoint_missing".to_owned(),
            detail: "post-restart lineage missing replay checkpoint".to_owned(),
        })
}

fn validate_replay_lineage(
    pre_restart: &[CanonicalCommitRecord],
    post_restart: &[CanonicalCommitRecord],
) -> Result<(), BlockPipelineError> {
    for (index, expected) in pre_restart.iter().enumerate() {
        let found = checkpoint_at(post_restart, index, expected)?;
        ensure_same_block_height(index, expected, found)?;
        ensure_same_producer_role(index, expected, found)?;
        ensure_same_payload_digest(index, expected, found)?;
        ensure_same_transaction_ids(index, expected, found)?;
    }
    Ok(())
}

fn checkpoint_at<'a>(
    post_restart: &'a [CanonicalCommitRecord],
    index: usize,
    expected: &CanonicalCommitRecord,
) -> Result<&'a CanonicalCommitRecord, BlockPipelineError> {
    post_restart
        .get(index)
        .ok_or_else(|| BlockPipelineError::ReplayDrift {
            reason_code: "canonical_replay_checkpoint_missing".to_owned(),
            detail: format!(
                "post-restart lineage missing checkpoint at index {index} (expected height {})",
                expected.block_height
            ),
        })
}

fn ensure_same_block_height(
    index: usize,
    expected: &CanonicalCommitRecord,
    found: &CanonicalCommitRecord,
) -> Result<(), BlockPipelineError> {
    if found.block_height == expected.block_height {
        return Ok(());
    }
    Err(BlockPipelineError::ReplayDrift {
        reason_code: "canonical_replay_block_height_mismatch".to_owned(),
        detail: format!(
            "canonical replay block height mismatch at index {index}: expected {}, found {}",
            expected.block_height, found.block_height
        ),
    })
}

fn ensure_same_producer_role(
    index: usize,
    expected: &CanonicalCommitRecord,
    found: &CanonicalCommitRecord,
) -> Result<(), BlockPipelineError> {
    if found.producer_role == expected.producer_role {
        return Ok(());
    }
    Err(BlockPipelineError::ReplayDrift {
        reason_code: "canonical_replay_producer_role_mismatch".to_owned(),
        detail: format!(
            "canonical replay producer role mismatch at index {index}: expected {}, found {}",
            expected.producer_role.as_str(),
            found.producer_role.as_str()
        ),
    })
}

fn ensure_same_payload_digest(
    index: usize,
    expected: &CanonicalCommitRecord,
    found: &CanonicalCommitRecord,
) -> Result<(), BlockPipelineError> {
    if found.payload_digest == expected.payload_digest {
        return Ok(());
    }
    Err(BlockPipelineError::ReplayDrift {
        reason_code: "canonical_replay_payload_digest_mismatch".to_owned(),
        detail: format!(
            "canonical replay payload digest mismatch at index {index}: expected {}, found {}",
            expected.payload_digest, found.payload_digest
        ),
    })
}

fn ensure_same_transaction_ids(
    index: usize,
    expected: &CanonicalCommitRecord,
    found: &CanonicalCommitRecord,
) -> Result<(), BlockPipelineError> {
    if found.transaction_ids == expected.transaction_ids {
        return Ok(());
    }
    Err(BlockPipelineError::ReplayDrift {
        reason_code: "canonical_replay_transaction_ids_mismatch".to_owned(),
        detail: format!(
            "canonical replay transaction ids mismatch at index {index}: expected {:?}, found {:?}",
            expected.transaction_ids, found.transaction_ids
        ),
    })
}
