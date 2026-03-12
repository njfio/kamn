use std::collections::{BTreeMap, BTreeSet};

use super::{ListenerQuorumDecision, ListenerQuorumError, ListenerQuorumInput};
use crate::runtime::runtime_phase_coordination::did_validation::parse_listener_did;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Listener quorum evaluator.
pub struct ListenerQuorumEvaluator {
    required_confirmations: usize,
    latest_sequence_by_event: BTreeMap<String, u64>,
}

impl ListenerQuorumEvaluator {
    /// Handles new.
    pub fn new(required_confirmations: usize) -> Result<Self, ListenerQuorumError> {
        if required_confirmations == 0 {
            return Err(ListenerQuorumError::InvalidRequiredConfirmations { required: required_confirmations });
        }
        Ok(Self { required_confirmations, latest_sequence_by_event: BTreeMap::new() })
    }

    /// Handles evaluate.
    pub fn evaluate(&mut self, input: ListenerQuorumInput) -> Result<ListenerQuorumDecision, ListenerQuorumError> {
        reject_replayed_sequence(&self.latest_sequence_by_event, &input)?;
        let confirmed_listeners = collect_confirmed_listeners(&input)?;
        if confirmed_listeners.len() < self.required_confirmations {
            return Err(ListenerQuorumError::InsufficientConfirmations {
                required: self.required_confirmations,
                received: confirmed_listeners.len(),
            });
        }
        self.latest_sequence_by_event.insert(input.event_id().to_owned(), input.event_sequence());
        Ok(ListenerQuorumDecision {
            event_id: input.event_id().to_owned(),
            event_sequence: input.event_sequence(),
            required_confirmations: self.required_confirmations,
            confirmed_listeners,
            accepted: true,
        })
    }
}

/// Handles evaluate daemon listener quorum.
pub fn evaluate_daemon_listener_quorum(
    evaluator: &mut ListenerQuorumEvaluator,
    input: ListenerQuorumInput,
) -> Result<ListenerQuorumDecision, ListenerQuorumError> {
    evaluator.evaluate(input)
}

fn reject_replayed_sequence(
    latest_sequence_by_event: &BTreeMap<String, u64>,
    input: &ListenerQuorumInput,
) -> Result<(), ListenerQuorumError> {
    if let Some(previous_sequence) = latest_sequence_by_event.get(input.event_id()) {
        if input.event_sequence() <= *previous_sequence {
            return Err(ListenerQuorumError::ReplayedEventSequence {
                event_id: input.event_id().to_owned(),
                previous_sequence: *previous_sequence,
                received_sequence: input.event_sequence(),
            });
        }
    }
    Ok(())
}

fn collect_confirmed_listeners(input: &ListenerQuorumInput) -> Result<Vec<String>, ListenerQuorumError> {
    let mut confirmed = BTreeSet::new();
    for attestation in input.attestations() {
        parse_listener_did(attestation.listener_did(), "attestations[].listener_did")?;
        if !confirmed.insert(attestation.listener_did().to_owned()) {
            return Err(ListenerQuorumError::DuplicateListenerAttestation {
                listener_did: attestation.listener_did().to_owned(),
            });
        }
    }
    Ok(confirmed.into_iter().collect())
}
