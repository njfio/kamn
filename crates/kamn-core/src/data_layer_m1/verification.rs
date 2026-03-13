use super::{
    support::{anchor_outcome_kind, is_valid_content_hash, leaf_digest, node_digest},
    DataLayerM1AnchorFailureMatrixCase, DataLayerM1AnchorFailureMatrixDecision,
    DataLayerM1AnchorFailureMatrixEvidence, DataLayerM1AnchorFailureMatrixReport,
    DataLayerM1Error, DataLayerM1MerkleInclusionProof, DataLayerM1MerkleLeaf,
    DataLayerM1ProofSiblingSide, DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_DRIFT_REASON_CODE,
    DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_STABLE_REASON_CODE,
    DATA_LAYER_M1_PROOF_VERIFICATION_INVALID_REASON_CODE,
    DATA_LAYER_M1_PROOF_VERIFICATION_VALID_REASON_CODE,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM1ProofVerificationDecision {
    Valid { reason_code: &'static str },
    Invalid { reason_code: &'static str, error: DataLayerM1Error },
}

pub fn verify_data_layer_m1_inclusion_proof(
    proof: &DataLayerM1MerkleInclusionProof,
) -> Result<(), DataLayerM1Error> {
    validate_proof_metadata(proof)?;
    validate_leaf_hash(proof)?;
    validate_root_hash(proof)
}

pub fn evaluate_data_layer_m1_inclusion_proof(
    proof: &DataLayerM1MerkleInclusionProof,
) -> DataLayerM1ProofVerificationDecision {
    match verify_data_layer_m1_inclusion_proof(proof) {
        Ok(()) => DataLayerM1ProofVerificationDecision::Valid {
            reason_code: DATA_LAYER_M1_PROOF_VERIFICATION_VALID_REASON_CODE,
        },
        Err(error) => DataLayerM1ProofVerificationDecision::Invalid {
            reason_code: DATA_LAYER_M1_PROOF_VERIFICATION_INVALID_REASON_CODE,
            error,
        },
    }
}

pub fn evaluate_data_layer_m1_anchor_failure_matrix(
    cases: &[DataLayerM1AnchorFailureMatrixCase],
) -> Result<DataLayerM1AnchorFailureMatrixReport, DataLayerM1Error> {
    if cases.is_empty() {
        return Err(DataLayerM1Error::InvalidFailureMatrixInput("cases"));
    }
    let evidence = collect_failure_matrix_evidence(cases)?;
    Ok(DataLayerM1AnchorFailureMatrixReport {
        decision: failure_matrix_decision(&evidence),
        evidence,
    })
}

fn validate_proof_metadata(proof: &DataLayerM1MerkleInclusionProof) -> Result<(), DataLayerM1Error> {
    require_non_empty(proof.batch_id.as_str(), "batch_id")?;
    require_non_empty(proof.merkle_root.as_str(), "merkle_root")?;
    require_non_empty(proof.message_id.as_str(), "message_id")?;
    if !is_valid_content_hash(proof.content_hash.as_str()) {
        return Err(DataLayerM1Error::InvalidContentHash(proof.content_hash.clone()));
    }
    Ok(())
}

fn require_non_empty(value: &str, field: &'static str) -> Result<(), DataLayerM1Error> {
    if value.trim().is_empty() {
        return Err(DataLayerM1Error::EmptyField(field));
    }
    Ok(())
}

fn validate_leaf_hash(proof: &DataLayerM1MerkleInclusionProof) -> Result<(), DataLayerM1Error> {
    let expected_leaf = leaf_digest(&DataLayerM1MerkleLeaf {
        message_id: proof.message_id.clone(),
        leaf_index: proof.leaf_index,
        content_hash: proof.content_hash.clone(),
    });
    if expected_leaf != proof.leaf_hash {
        return Err(DataLayerM1Error::InvalidMerkleProof("leaf hash mismatch"));
    }
    Ok(())
}

fn validate_root_hash(proof: &DataLayerM1MerkleInclusionProof) -> Result<(), DataLayerM1Error> {
    let current = proof.steps.iter().enumerate().try_fold(proof.leaf_hash.clone(), |hash, step| {
        fold_proof_step(step.0, step.1, &hash)
    })?;
    if current != proof.merkle_root {
        return Err(DataLayerM1Error::InvalidMerkleProof("proof root mismatch"));
    }
    Ok(())
}

fn fold_proof_step(
    level_index: usize,
    step: &super::DataLayerM1MerkleProofStep,
    current: &str,
) -> Result<String, DataLayerM1Error> {
    require_non_empty(step.sibling_hash.as_str(), "sibling_hash")
        .map_err(|_| DataLayerM1Error::InvalidMerkleProof("proof sibling hash must not be empty"))?;
    Ok(match step.sibling_side {
        DataLayerM1ProofSiblingSide::Left => node_digest(level_index, step.sibling_hash.as_str(), current),
        DataLayerM1ProofSiblingSide::Right => node_digest(level_index, current, step.sibling_hash.as_str()),
    })
}

fn collect_failure_matrix_evidence(
    cases: &[DataLayerM1AnchorFailureMatrixCase],
) -> Result<Vec<DataLayerM1AnchorFailureMatrixEvidence>, DataLayerM1Error> {
    cases.iter().map(build_failure_matrix_entry).collect()
}

fn build_failure_matrix_entry(
    case: &DataLayerM1AnchorFailureMatrixCase,
) -> Result<DataLayerM1AnchorFailureMatrixEvidence, DataLayerM1Error> {
    require_non_empty(case.case_id.as_str(), "case_id")?;
    let observed_outcome_kind = anchor_outcome_kind(&case.result.outcome);
    let observed_retry_class = case.result.retry_class;
    Ok(DataLayerM1AnchorFailureMatrixEvidence {
        case_id: case.case_id.clone(),
        batch_id: case.result.batch_id.clone(),
        idempotency_key: case.result.idempotency_key.clone(),
        expected_retry_class: case.expected_retry_class,
        observed_retry_class,
        expected_outcome_kind: case.expected_outcome_kind,
        observed_outcome_kind,
        mismatch: observed_retry_class != case.expected_retry_class
            || observed_outcome_kind != case.expected_outcome_kind,
    })
}

fn failure_matrix_decision(
    evidence: &[DataLayerM1AnchorFailureMatrixEvidence],
) -> DataLayerM1AnchorFailureMatrixDecision {
    if evidence.iter().all(|entry| !entry.mismatch) {
        DataLayerM1AnchorFailureMatrixDecision::Stable {
            reason_code: DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_STABLE_REASON_CODE,
        }
    } else {
        DataLayerM1AnchorFailureMatrixDecision::DriftDetected {
            reason_code: DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_DRIFT_REASON_CODE,
        }
    }
}
