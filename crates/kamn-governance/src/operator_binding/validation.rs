use crate::operator_binding::constants::CANONICAL_PROOF_TYPE;
use crate::operator_binding::{OperatorBindingError, OperatorBindingProof};

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), OperatorBindingError> {
    if value.trim().is_empty() {
        return Err(OperatorBindingError::EmptyProofField(field));
    }
    Ok(())
}

pub(super) fn validate_proof(
    proof: &OperatorBindingProof,
    operator_did: &str,
) -> Result<(), OperatorBindingError> {
    validate_non_empty("type_name", &proof.type_name)?;
    validate_non_empty("created", &proof.created)?;
    validate_non_empty("verification_method", &proof.verification_method)?;
    validate_non_empty("proof_value", &proof.proof_value)?;

    if proof.type_name != CANONICAL_PROOF_TYPE {
        return Err(OperatorBindingError::InvalidProofType(
            proof.type_name.clone(),
        ));
    }

    let expected_prefix = format!("{operator_did}#");
    if !proof.verification_method.starts_with(&expected_prefix) {
        return Err(OperatorBindingError::ProofVerificationMethodMismatch {
            expected_prefix,
            actual: proof.verification_method.clone(),
        });
    }
    Ok(())
}
