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

/// Deterministic proof-verification decision wrapper with stable reason markers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM1ProofVerificationDecision {
    /// Proof verified successfully.
    Valid {
        /// Stable reason code.
        reason_code: &'static str,
    },
    /// Proof verification failed.
    Invalid {
        /// Stable reason code.
        reason_code: &'static str,
        /// Original fail-closed verification error.
        error: DataLayerM1Error,
    },
}

/// Verifies an inclusion proof fail-closed.
pub fn verify_data_layer_m1_inclusion_proof(
    proof: &DataLayerM1MerkleInclusionProof,
) -> Result<(), DataLayerM1Error> {
    if proof.batch_id.trim().is_empty() {
        return Err(DataLayerM1Error::EmptyField("batch_id"));
    }
    if proof.merkle_root.trim().is_empty() {
        return Err(DataLayerM1Error::EmptyField("merkle_root"));
    }
    if proof.message_id.trim().is_empty() {
        return Err(DataLayerM1Error::EmptyField("message_id"));
    }
    if !is_valid_content_hash(proof.content_hash.as_str()) {
        return Err(DataLayerM1Error::InvalidContentHash(proof.content_hash.clone()));
    }

    let expected_leaf = leaf_digest(&DataLayerM1MerkleLeaf {
        message_id: proof.message_id.clone(),
        leaf_index: proof.leaf_index,
        content_hash: proof.content_hash.clone(),
    });
    if expected_leaf != proof.leaf_hash {
        return Err(DataLayerM1Error::InvalidMerkleProof("leaf hash mismatch"));
    }

    let mut current = proof.leaf_hash.clone();
    for (level_index, step) in proof.steps.iter().enumerate() {
        if step.sibling_hash.trim().is_empty() {
            return Err(DataLayerM1Error::InvalidMerkleProof(
                "proof sibling hash must not be empty",
            ));
        }
        current = match step.sibling_side {
            DataLayerM1ProofSiblingSide::Left => {
                node_digest(level_index, step.sibling_hash.as_str(), current.as_str())
            }
            DataLayerM1ProofSiblingSide::Right => {
                node_digest(level_index, current.as_str(), step.sibling_hash.as_str())
            }
        };
    }

    if current != proof.merkle_root {
        return Err(DataLayerM1Error::InvalidMerkleProof("proof root mismatch"));
    }
    Ok(())
}

/// Evaluates proof verification and projects a deterministic reason-coded decision.
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

/// Evaluates deterministic anchoring failure-matrix expectations.
pub fn evaluate_data_layer_m1_anchor_failure_matrix(
    cases: &[DataLayerM1AnchorFailureMatrixCase],
) -> Result<DataLayerM1AnchorFailureMatrixReport, DataLayerM1Error> {
    if cases.is_empty() {
        return Err(DataLayerM1Error::InvalidFailureMatrixInput("cases"));
    }

    let mut evidence = Vec::with_capacity(cases.len());
    for case in cases {
        if case.case_id.trim().is_empty() {
            return Err(DataLayerM1Error::InvalidFailureMatrixInput("case_id"));
        }
        let observed_outcome_kind = anchor_outcome_kind(&case.result.outcome);
        let observed_retry_class = case.result.retry_class;
        let mismatch = observed_retry_class != case.expected_retry_class
            || observed_outcome_kind != case.expected_outcome_kind;
        evidence.push(DataLayerM1AnchorFailureMatrixEvidence {
            case_id: case.case_id.clone(),
            batch_id: case.result.batch_id.clone(),
            idempotency_key: case.result.idempotency_key.clone(),
            expected_retry_class: case.expected_retry_class,
            observed_retry_class,
            expected_outcome_kind: case.expected_outcome_kind,
            observed_outcome_kind,
            mismatch,
        });
    }

    let decision = if evidence.iter().all(|entry| !entry.mismatch) {
        DataLayerM1AnchorFailureMatrixDecision::Stable {
            reason_code: DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_STABLE_REASON_CODE,
        }
    } else {
        DataLayerM1AnchorFailureMatrixDecision::DriftDetected {
            reason_code: DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_DRIFT_REASON_CODE,
        }
    };

    Ok(DataLayerM1AnchorFailureMatrixReport { decision, evidence })
}
