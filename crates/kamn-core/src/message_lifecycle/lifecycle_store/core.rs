use super::validation::{is_expirable_status, is_valid_transition, validate_registration_request};
use super::MessageLifecycleStore;
use crate::message_lifecycle::{MessageLifecycleError, MessageProofAdmissionError, MessageStatus};
use crate::{
    ProcessorProofAdmissionEvaluator, ProcessorProofAdmissionInput, ProcessorProofArtifact,
};

impl MessageLifecycleStore {
    /// Registers a new message with sender/recipient metadata and lifecycle timestamps.
    pub fn register(
        &mut self,
        message_id: &str,
        sender: &str,
        recipients: Vec<String>,
        created: &str,
        expires: &str,
    ) -> Result<(), MessageLifecycleError> {
        validate_registration_request(message_id, sender, &recipients, created, expires)?;
        self.ensure_message_id_is_new(message_id)?;
        self.insert_registered_message(message_id, sender, recipients, created, expires);
        Ok(())
    }

    /// Returns the current status for a message id.
    pub fn status(&self, message_id: &str) -> Result<MessageStatus, MessageLifecycleError> {
        self.records
            .get(message_id)
            .map(|record| record.status)
            .ok_or_else(|| MessageLifecycleError::NotFound(message_id.to_owned()))
    }

    /// Applies a lifecycle transition when the edge is valid under policy.
    pub fn transition(
        &mut self,
        message_id: &str,
        to: MessageStatus,
    ) -> Result<(), MessageLifecycleError> {
        let from = self.status(message_id)?;
        if !is_valid_transition(from, to) {
            return Err(MessageLifecycleError::InvalidTransition { from, to });
        }
        self.apply_status(message_id, to)
    }

    /// Expires one message when `observed_at` is after the stored expiry timestamp.
    pub fn expire_message_if_overdue(
        &mut self,
        message_id: &str,
        observed_at: &str,
    ) -> Result<bool, MessageLifecycleError> {
        self.ensure_observed_at(observed_at)?;
        let record = self.require_record(message_id)?;
        if !is_expirable_status(record.status) || observed_at <= record.expires.as_str() {
            return Ok(false);
        }
        self.transition(message_id, MessageStatus::Expired)?;
        Ok(true)
    }

    /// Expires all active messages that are overdue at `observed_at`.
    pub fn expire_overdue_messages(
        &mut self,
        observed_at: &str,
    ) -> Result<Vec<String>, MessageLifecycleError> {
        self.ensure_observed_at(observed_at)?;
        let overdue_ids = self.collect_overdue_ids(observed_at);
        for message_id in &overdue_ids {
            self.transition(message_id, MessageStatus::Expired)?;
        }
        Ok(overdue_ids)
    }

    /// Validates processor proof evidence for a delivered message and transitions it to validated.
    pub fn validate_with_processor_proof(
        &mut self,
        message_id: &str,
        expected_payload_commitment: &str,
        artifact: ProcessorProofArtifact,
        evaluator: &mut ProcessorProofAdmissionEvaluator,
    ) -> Result<(), MessageProofAdmissionError> {
        self.ensure_delivered_state(message_id)?;
        let input =
            ProcessorProofAdmissionInput::new(message_id, expected_payload_commitment, artifact)
                .map_err(MessageProofAdmissionError::Proof)?;
        evaluator
            .evaluate(input)
            .map_err(MessageProofAdmissionError::Proof)?;
        self.transition(message_id, MessageStatus::Validated)
            .map_err(MessageProofAdmissionError::Lifecycle)
    }

    pub(super) fn apply_status(
        &mut self,
        message_id: &str,
        to: MessageStatus,
    ) -> Result<(), MessageLifecycleError> {
        let from = self.require_record(message_id)?.status;
        if from == to {
            return Ok(());
        }
        self.update_record_status(message_id, to)?;
        self.reindex_status(message_id, from, to);
        Ok(())
    }

    fn ensure_observed_at(&self, observed_at: &str) -> Result<(), MessageLifecycleError> {
        if observed_at.trim().is_empty() {
            return Err(MessageLifecycleError::EmptyTimestamp("observed_at"));
        }
        Ok(())
    }

    fn collect_overdue_ids(&self, observed_at: &str) -> Vec<String> {
        self.records
            .iter()
            .filter(|&(_, record)| {
                is_expirable_status(record.status) && observed_at > record.expires.as_str()
            })
            .map(|(message_id, _)| message_id.clone())
            .collect()
    }

    pub(super) fn collect_index_values(
        &self,
        ids: Option<&std::collections::BTreeSet<String>>,
    ) -> Vec<String> {
        ids.map(|values| values.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub(super) fn require_record(
        &self,
        message_id: &str,
    ) -> Result<&super::MessageRecord, MessageLifecycleError> {
        self.records
            .get(message_id)
            .ok_or_else(|| MessageLifecycleError::NotFound(message_id.to_owned()))
    }

    fn ensure_delivered_state(&self, message_id: &str) -> Result<(), MessageProofAdmissionError> {
        let status = self
            .status(message_id)
            .map_err(MessageProofAdmissionError::Lifecycle)?;
        if status != MessageStatus::Delivered {
            return Err(MessageProofAdmissionError::InvalidValidationState { found: status });
        }
        Ok(())
    }
}
