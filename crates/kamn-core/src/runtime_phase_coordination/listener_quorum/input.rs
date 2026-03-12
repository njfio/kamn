use super::{ListenerAttestation, ListenerQuorumError};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Listener quorum input.
pub struct ListenerQuorumInput {
    event_id: String,
    event_sequence: u64,
    attestations: Vec<ListenerAttestation>,
}

impl ListenerQuorumInput {
    /// Handles new.
    pub fn new(event_id: &str, event_sequence: u64, attestations: Vec<ListenerAttestation>) -> Result<Self, ListenerQuorumError> {
        if event_id.trim().is_empty() { return Err(ListenerQuorumError::InvalidEventId); }
        if event_sequence == 0 { return Err(ListenerQuorumError::InvalidEventSequence); }
        Ok(Self { event_id: event_id.to_owned(), event_sequence, attestations })
    }

    /// Handles event id.
    pub fn event_id(&self) -> &str { &self.event_id }

    /// Handles event sequence.
    pub fn event_sequence(&self) -> u64 { self.event_sequence }

    /// Handles attestations.
    pub fn attestations(&self) -> &[ListenerAttestation] { &self.attestations }
}
