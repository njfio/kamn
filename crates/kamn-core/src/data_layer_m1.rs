//! M1 trust-anchor contracts for merkle batching, proof generation, and Kolme anchoring.
//!
//! This module builds on M0 content-hash records and provides deterministic
//! merkle root assembly, inclusion-proof verification, and an idempotent
//! anchoring worker that targets the existing Kolme runtime-commit client.

mod anchoring;
mod batch;
mod models;
mod support;
#[cfg(test)]
mod tests;
mod verification;

pub use anchoring::DataLayerM1KolmeAnchoringWorker;
pub use batch::DataLayerM1MerkleBatch;
pub use models::{
    DataLayerM1AnchorFailureMatrixCase, DataLayerM1AnchorFailureMatrixDecision,
    DataLayerM1AnchorFailureMatrixEvidence, DataLayerM1AnchorFailureMatrixReport,
    DataLayerM1AnchorOutcome, DataLayerM1AnchorOutcomeKind, DataLayerM1AnchorReceipt,
    DataLayerM1AnchorResult, DataLayerM1AnchorRetryClass, DataLayerM1Error,
    DataLayerM1MerkleInclusionProof, DataLayerM1MerkleLeaf, DataLayerM1MerkleProofStep,
    DataLayerM1ProofSiblingSide, DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_DRIFT_REASON_CODE,
    DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_STABLE_REASON_CODE, DATA_LAYER_M1_HASH_ALGORITHM,
    DATA_LAYER_M1_PROOF_VERIFICATION_INVALID_REASON_CODE,
    DATA_LAYER_M1_PROOF_VERIFICATION_VALID_REASON_CODE,
};
pub use verification::{
    evaluate_data_layer_m1_anchor_failure_matrix, evaluate_data_layer_m1_inclusion_proof,
    verify_data_layer_m1_inclusion_proof, DataLayerM1ProofVerificationDecision,
};
