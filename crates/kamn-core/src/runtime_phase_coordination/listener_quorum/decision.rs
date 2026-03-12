#[derive(Debug, Clone, PartialEq, Eq)]
/// Listener quorum decision.
pub struct ListenerQuorumDecision {
    /// Event id.
    pub event_id: String,
    /// Event sequence.
    pub event_sequence: u64,
    /// Required confirmations.
    pub required_confirmations: usize,
    /// Confirmed listeners.
    pub confirmed_listeners: Vec<String>,
    /// Accepted.
    pub accepted: bool,
}
