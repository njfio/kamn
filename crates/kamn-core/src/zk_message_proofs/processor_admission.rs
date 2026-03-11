use super::errors::{require_non_empty_artifact_field, ZkDesignError};

/// Processor-side proof artifact accepted for admission checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessorProofArtifact {
    pub artifact_id: String,
    pub message_id: String,
    pub payload_commitment: String,
    pub proof_value: String,
}

impl ProcessorProofArtifact {
    pub fn new(
        artifact_id: &str,
        message_id: &str,
        payload_commitment: &str,
        proof_value: &str,
    ) -> Result<Self, ZkDesignError> {
        require_non_empty_artifact_field("artifact_id", artifact_id)?;
        require_non_empty_artifact_field("message_id", message_id)?;
        require_non_empty_artifact_field("payload_commitment", payload_commitment)?;
        require_non_empty_artifact_field("proof_value", proof_value)?;
        validate_commitment(payload_commitment)?;
        validate_proof_value(proof_value)?;
        Ok(Self {
            artifact_id: artifact_id.to_owned(),
            message_id: message_id.to_owned(),
            payload_commitment: payload_commitment.to_owned(),
            proof_value: proof_value.to_owned(),
        })
    }
}

/// Processor admission input combining the message and its proof artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessorProofAdmissionInput {
    pub message_id: String,
    pub expected_payload_commitment: String,
    pub artifact: ProcessorProofArtifact,
}

impl ProcessorProofAdmissionInput {
    pub fn new(
        message_id: &str,
        expected_payload_commitment: &str,
        artifact: ProcessorProofArtifact,
    ) -> Result<Self, ZkDesignError> {
        require_non_empty_artifact_field("message_id", message_id)?;
        require_non_empty_artifact_field(
            "expected_payload_commitment",
            expected_payload_commitment,
        )?;
        Ok(Self {
            message_id: message_id.to_owned(),
            expected_payload_commitment: expected_payload_commitment.to_owned(),
            artifact,
        })
    }
}

/// Stable processor admission result returned after validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessorProofAdmissionDecision {
    pub message_id: String,
    pub artifact_id: String,
    pub payload_commitment: String,
}

/// Stateful evaluator that rejects replayed processor proof artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProcessorProofAdmissionEvaluator {
    accepted_artifact_ids: std::collections::BTreeSet<String>,
}

impl ProcessorProofAdmissionEvaluator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn evaluate(
        &mut self,
        input: ProcessorProofAdmissionInput,
    ) -> Result<ProcessorProofAdmissionDecision, ZkDesignError> {
        validate_message_match(&input)?;
        validate_commitment_match(&input)?;
        validate_deterministic_proof(&input.artifact)?;
        if !self
            .accepted_artifact_ids
            .insert(input.artifact.artifact_id.clone())
        {
            return Err(ZkDesignError::ProofArtifactReplay(
                input.artifact.artifact_id,
            ));
        }
        Ok(ProcessorProofAdmissionDecision {
            message_id: input.message_id,
            artifact_id: input.artifact.artifact_id,
            payload_commitment: input.expected_payload_commitment,
        })
    }
}

fn validate_commitment(payload_commitment: &str) -> Result<(), ZkDesignError> {
    if !payload_commitment.starts_with("fnv1a64:") {
        return Err(ZkDesignError::InvalidProofArtifact(
            "payload_commitment must start with `fnv1a64:`".to_owned(),
        ));
    }
    if payload_commitment == "fnv1a64:" {
        return Err(ZkDesignError::InvalidProofArtifact(
            "payload_commitment must include digest bytes".to_owned(),
        ));
    }
    Ok(())
}

fn validate_proof_value(proof_value: &str) -> Result<(), ZkDesignError> {
    if !proof_value.starts_with("proof:") {
        return Err(ZkDesignError::InvalidProofArtifact(
            "proof_value must start with `proof:`".to_owned(),
        ));
    }
    Ok(())
}

fn validate_message_match(input: &ProcessorProofAdmissionInput) -> Result<(), ZkDesignError> {
    if input.artifact.message_id == input.message_id {
        return Ok(());
    }
    Err(ZkDesignError::ProofArtifactMessageMismatch {
        expected: input.message_id.clone(),
        found: input.artifact.message_id.clone(),
    })
}

fn validate_commitment_match(input: &ProcessorProofAdmissionInput) -> Result<(), ZkDesignError> {
    if input.artifact.payload_commitment == input.expected_payload_commitment {
        return Ok(());
    }
    Err(ZkDesignError::ProofArtifactCommitmentMismatch {
        expected: input.expected_payload_commitment.clone(),
        found: input.artifact.payload_commitment.clone(),
    })
}

fn validate_deterministic_proof(artifact: &ProcessorProofArtifact) -> Result<(), ZkDesignError> {
    let expected = format!("proof:ok:{}", artifact.artifact_id);
    if artifact.proof_value == expected {
        return Ok(());
    }
    Err(ZkDesignError::ProofVerificationFailed {
        artifact_id: artifact.artifact_id.clone(),
        reason: "proof value failed deterministic verification".to_owned(),
    })
}
