use super::ValidatorProofConsensusStatus;

pub(super) fn consensus_status(
    valid: usize,
    invalid: usize,
    replay: usize,
) -> ValidatorProofConsensusStatus {
    if consensus_bucket_count(valid, invalid, replay) > 1 {
        ValidatorProofConsensusStatus::ValidatorMismatch
    } else if valid > 0 {
        ValidatorProofConsensusStatus::ConsensusValid
    } else if invalid > 0 {
        ValidatorProofConsensusStatus::ConsensusInvalid
    } else {
        ValidatorProofConsensusStatus::ConsensusReplay
    }
}

fn consensus_bucket_count(valid: usize, invalid: usize, replay: usize) -> usize {
    [valid, invalid, replay]
        .into_iter()
        .filter(|count| *count > 0)
        .count()
}
