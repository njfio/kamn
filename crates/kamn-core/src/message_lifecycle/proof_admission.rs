use super::lifecycle_errors::MessageProofAdmissionError;
use super::lifecycle_store::MessageLifecycleStore;
use super::lifecycle_types::MessageStatus;
use crate::{
    ProcessorProofAdmissionEvaluator, ProcessorProofAdmissionInput, ProcessorProofArtifact,
};

impl MessageLifecycleStore {
    /// Validates processor proof evidence for a delivered message and transitions it to validated.
    pub fn validate_with_processor_proof(
        &mut self,
        message_id: &str,
        expected_payload_commitment: &str,
        artifact: ProcessorProofArtifact,
        evaluator: &mut ProcessorProofAdmissionEvaluator,
    ) -> Result<(), MessageProofAdmissionError> {
        let status = self
            .status(message_id)
            .map_err(MessageProofAdmissionError::Lifecycle)?;
        if status != MessageStatus::Delivered {
            return Err(MessageProofAdmissionError::InvalidValidationState { found: status });
        }

        let input =
            ProcessorProofAdmissionInput::new(message_id, expected_payload_commitment, artifact)
                .map_err(MessageProofAdmissionError::Proof)?;
        evaluator
            .evaluate(input)
            .map_err(MessageProofAdmissionError::Proof)?;

        self.transition(message_id, MessageStatus::Validated)
            .map_err(MessageProofAdmissionError::Lifecycle)?;
        Ok(())
    }
}
