use crate::block_pipeline::block_pipeline_support::{
    CanonicalCommitRecord, CanonicalReplayEvidenceBundle,
};
use crate::block_pipeline::models::BlockPipelineError;

const CANONICAL_REPLAY_EVIDENCE_SCHEMA_VERSION: &str = "kamn.runtime.canonical-replay-evidence.v1";

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
